// SPDX-License-Identifier: AGPL-3.0-or-later
main()
{
    a = [];
    a["score"] = 20;
    a[0] = 2;
    if (1)
    {
        privateAlias = a;
    }
    a[1] = 20;
    b = patch(a);
    a[0] = undefined;
    return a["score"] + b[0] + b[1];
}

patch(a)
{
    a[1] = 20;
    return a;
}
