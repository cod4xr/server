// Missing script parameters contain undefined; isdefined checks the value's tag.
main()
{
    return optional();
}

optional(value)
{
    if (isdefined(value))
    {
        return 0;
    }
    return 1;
}
