module rsph.core.result;

public import std.sumtype : match, tryMatch;

import std.sumtype : SumType;

/// Value returned by a successful operation.
struct Success(T)
{
    static if (!is(T == void))
        T value;
}

/// Error returned by a failed operation.
struct Failure(E)
{
    static if (!is(E == void))
        E error;
}

/// A typed value or error. The wrappers distinguish the cases even when T and E coincide.
alias Result(T, E) = SumType!(Success!T, Failure!E);
