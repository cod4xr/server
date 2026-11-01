main()
{
    root = spawnstruct();
    root.child = spawnstruct();
    old = root.child;
    old.value = 1;
    root.child.value = replace_child(root);
    return old.value + 10 * root.child.value;
}

replace_child(root)
{
    root.child = spawnstruct();
    return 7;
}
