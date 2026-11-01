// SPDX-License-Identifier: AGPL-3.0-or-later
main()
{
    state = spawnstruct();
    state.text = "";
    state.anchor = &"MENU";
    for (index = 0; index < 1000; index++)
    {
        demo\library::append(state);
    }
    return state.text.size + isdefined(state.anchor);
}
