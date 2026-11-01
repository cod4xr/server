// Read-only host-origin arrays; no implicit world, construction or indexed writes.
main(items)
{
    return read(items) + items.size;
}

read(items)
{
    return items[0] + items["bonus"];
}
