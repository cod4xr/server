// Default developer mode returns 20; --gsc-developer enabled returns 42.
main()
{
    score = 20;
    /#
        score += 22;
    #/
    return score;
}
