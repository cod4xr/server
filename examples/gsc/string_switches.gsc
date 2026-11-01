// Plain strings and integers remain separate switch domains.
main()
{
    return character_score("ranger") + prestige_score(2);
}

character_score(name)
{
    switch (name)
    {
        case "ranger": return 22;
        case "scout": return 10;
        default: return 0;
    }
}

prestige_score(prestige)
{
    switch (prestige)
    {
        case "2": return 2;
        case 2: return 20;
        default: return 0;
    }
}
