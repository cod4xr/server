// Plain-string indexing returns bytes; size counts bytes before the first NUL.
main()
{
    text = "Rust";
    if (text.size != 4)
    {
        return undefined;
    }
    return text[0] + text[3];
}
