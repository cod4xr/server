// SPDX-License-Identifier: AGPL-3.0-or-later
main()
{
    a = [];
    if (1)
    {
        privateAlias = a;
    }
    i = 0;
    while (i < 2)
    {
        switch (i)
        {
        case 0:
            scratch = a;
            i++;
            continue;
        default:
            other = a;
            break;
        }
        i++;
    }
    return relay(a) + i;
}

relay(a)
{
    if (1)
    {
        privateAlias = a;
    }
    return a.size + 40;
}
