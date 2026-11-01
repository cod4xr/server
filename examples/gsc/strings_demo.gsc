// SPDX-License-Identifier: AGPL-3.0-or-later
main()
{
    value = "Rust";
    for (index = 0; index < 3; index++)
    {
        value = decorate(value);
    }
    return value;
}

decorate(value)
{
    return "[" + value + "]";
}
