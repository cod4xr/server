main()
{
    value = spawnstruct("evaluated extra argument", (1, 2, 3));
    return isdefined(value) + value.size;
}
