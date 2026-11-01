// SPDX-License-Identifier: AGPL-3.0-or-later
main()
{
    message = &"MENU_EXAMPLE";
    if (isstring(message))
    {
        return undefined;
    }
    return copy(message);
}

copy(value)
{
    return value;
}
