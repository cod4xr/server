// SPDX-License-Identifier: AGPL-3.0-or-later
// Run with: cargo run -p cod4x-server -- --execute-gsc examples/gsc/primitive_demo.gsc
// Expected stdout: integer 18

main()
{
    calculate = ::score_rounds;
    return [[calculate]](6);
}

score_rounds(round_count)
{
    result = 0;
    for (round = 0; round < round_count; round++)
    {
        switch (round)
        {
            case 0:
                continue;
            case 1:
                result += 10;
                break;
            default:
                result += 2;
        }
    }
    if (isint(result))
    {
        return result;
    }
    return 0;
}
