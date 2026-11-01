// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Audit reuse must survive checked mutations and partial string-collection failure without crossing store bindings."
)]
use crate::{array_read_support::number, source_developer_support::compile};
use cod4x_script::binding::LegacyDeveloperMode::Enabled;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

#[test]
fn context_reuse_keeps_one_audit_and_rebinding_another_store_requires_its_own_audit() {
    let module = compile(
        b"main(){a=[];a=undefined;b=[];b=undefined;c=[];return 42;}",
        Enabled,
    );
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let budget = ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        ..ObjectHeapBudget::default()
    };
    let mut first = ObjectHeap::new(&strings, budget).expect("First");
    let mut second = ObjectHeap::new(&strings, budget).expect("Second");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut first, &[]);
    for _ in 0..2 {
        assert_eq!(
            module
                .program()
                .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
                .expect("Reusable invocation")
                .value,
            number(42)
        );
    }
    assert_eq!(context.usage().array_owner_audits, 1);
    assert!(context.usage().array_owner_audit_reuses >= 1);
    let mut context = context.with_objects(&mut second, &[]);
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("New object namespace")
            .value,
        number(42)
    );
    assert_eq!(context.usage().array_owner_audits, 2);
    drop(context);
    assert_eq!(first.usage().live_array_values, 0);
    assert_eq!(second.usage().live_array_values, 0);
}

#[test]
fn completed_object_audit_survives_string_failure_then_checked_repair_and_cached_array_collection()
{
    let first = compile(b"main(o){return spawnstruct();}", Enabled);
    let repair = compile(
        b"main(o){o.text=undefined;a=[];a[0]=7;a=undefined;b=[];b=undefined;c=[];return 42;}",
        Enabled,
    );
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let stale = strings
        .allocate(b"PrivatePayload", work)
        .expect("Initially valid")
        .id;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 3,
            max_object_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let host = objects.allocate(work).expect("Host").id;
    objects
        .put(host, b"text", KernelValue::String(stale), &strings, work)
        .expect("Valid before independent string reclamation");
    objects.allocate(work).expect("Garbage one");
    objects.allocate(work).expect("Garbage two");
    strings
        .reclaim(&[], work)
        .expect("Deliberately stale stored string");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let error = first
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Object(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Later string phase fails");
    assert_eq!(error.cause.kind, KernelErrorKind::StaleStringReference);
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (1, 0)
    );
    assert_eq!(context.usage().array_owner_audits, 1);
    assert_eq!(
        repair
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Object(host)],
                KernelCallBudget::default(),
                &mut context
            )
            .expect("Checked repair before next pressure retry")
            .value,
        number(42)
    );
    assert_eq!(context.usage().array_owner_audits, 1);
    assert_eq!(context.usage().array_owner_audit_reuses, 1);
    assert_eq!(context.usage().collections, 1);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        0
    );
}
