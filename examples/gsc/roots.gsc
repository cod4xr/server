main()
{
    level.total = 10;
    self.total = 20;
    anim.total = 30;
    owned = game;
    return worker(owned);
}

worker(value)
{
    return level.total + self.total + anim.total + isdefined(value);
}
