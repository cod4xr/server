// Independent empty data can be copied, read and returned; indexed writes stay gated.
main()
{
    first = [];
    second = copy(first);
    return first.size + second.size + isdefined(second);
}

copy(value)
{
    return value;
}
