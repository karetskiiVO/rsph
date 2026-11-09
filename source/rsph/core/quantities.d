module rsph.core.quantities;

import std.math : sqrt;
import std.traits : isFloatingPoint, isIntegral;

/// Additive identity. Custom quantities supply a typed ZERO constant.
template zero(T)
{
    static if (isFloatingPoint!T)
        enum T zero = 0;
    else
        enum zero = T.ZERO;
}

/// Square root for a floating-point value or a custom quantity.
auto squareRoot(T)(T value)
if (isFloatingPoint!T || __traits(compiles, value.sqrt()))
{
    static if (isFloatingPoint!T)
        return sqrt(value);
    else
        return value.sqrt();
}

/// Convert a scalar into the domain's quantity type.
Target quantityFrom(Target, Source)(Source value)
if (preservesFloatPrecision!(Source, Target) &&
    (is(Source : Target) || canConstructQuantity!(Target, Source)))
{
    static if (is(Source : Target))
        return value;
    else
        return Target(value);
}

/// Time with an explicit scalar representation.
struct Time(T)
{
    T value = quantityDefault!T;

    auto opCmp(const Time other) const
    {
        return compareValues(value, other.value);
    }

    L opBinaryRight(string op, L)(L lhs) const
    if ((op == "*" || op == "/") && canScaleWith!(L, T))
    {
        ScalarOf!L duration = value;
        return mixin("lhs " ~ op ~ " duration");
    }
}

/// Mass with an explicit scalar representation.
struct Mass(T)
{
    T value = quantityDefault!T;

    auto opCmp(const Mass other) const
    {
        return compareValues(value, other.value);
    }

    S opBinary(string op : "*", S)(S weight) const
    if (isFloatingPoint!S && canScaleWith!(S, T))
    {
        S mass = value;
        return mass * weight;
    }

    L opBinaryRight(string op : "/", L)(L lhs) const
    if (canScaleWith!(L, T))
    {
        ScalarOf!L mass = value;
        return lhs / mass;
    }
}

/// Three-component value vector for positions, displacements and rates.
struct Vector3(T)
if (isFloatingPoint!T)
{
    T x = 0;
    T y = 0;
    T z = 0;

    enum ZERO = Vector3(0, 0, 0);

    Vector3 opBinary(string op)(const Vector3 rhs) const
    if (op == "+" || op == "-")
    {
        return Vector3(mixin("x " ~ op ~ " rhs.x"),
                       mixin("y " ~ op ~ " rhs.y"),
                       mixin("z " ~ op ~ " rhs.z"));
    }

    Vector3 opBinary(string op)(T scalar) const
    if (op == "*" || op == "/")
    {
        return Vector3(mixin("x " ~ op ~ " scalar"),
                       mixin("y " ~ op ~ " scalar"),
                       mixin("z " ~ op ~ " scalar"));
    }
}

alias Vec3 = Vector3!float;
alias DVec3 = Vector3!double;

private auto compareValues(T)(T left, T right)
{
    static if (isFloatingPoint!T)
    {
        // A NaN comparison stays unordered when D compares the result to zero.
        if (left != left || right != right)
            return T.nan;
        return cast(T) ((left > right) - (left < right));
    }
    else static if (__traits(compiles, left.opCmp(right)))
        return left.opCmp(right);
    else
        return (left > right) - (left < right);
}

private template quantityDefault(T)
{
    static if (isFloatingPoint!T)
        enum T quantityDefault = 0;
    else
        enum T quantityDefault = T.init;
}

private template ScalarOf(T)
{
    static if (is(T == Vector3!S, S))
        alias ScalarOf = S;
    else
        alias ScalarOf = T;
}

private enum bool canScaleWith(V, S) =
    isFloatingPoint!(ScalarOf!V) && is(S : ScalarOf!V) &&
    preservesFloatPrecision!(S, ScalarOf!V);

private template canConstructQuantity(Target, Source)
{
    static if (is(Target == Time!Scalar, Scalar))
        enum canConstructQuantity = is(Source == Scalar);
    else
        enum canConstructQuantity = __traits(compiles, Target(Source.init));
}

private template preservesFloatPrecision(From, To)
{
    static if (isFloatingPoint!To && isFloatingPoint!From)
        enum preservesFloatPrecision = From.mant_dig <= To.mant_dig &&
            From.max_exp <= To.max_exp;
    else static if (isFloatingPoint!To && isIntegral!From)
        enum preservesFloatPrecision = From.sizeof * 8 <= To.mant_dig;
    else
        enum preservesFloatPrecision = true;
}

unittest
{
    assert(Time!float.init.value == 0);
    assert(Mass!double.init.value == 0);
    assert(Time!double(1) < Time!double(2));
    assert(!(Time!double(double.nan) <= Time!double(2)));
    assert(Time!double(double.nan) != Time!double(double.nan));

    struct PartialValue
    {
        double opCmp(const PartialValue) const
        {
            return double.nan;
        }
    }

    assert(!(Time!PartialValue() <= Time!PartialValue()));
}

unittest
{
    const vector = DVec3(4, -8, 12);
    const time = Time!float(2);
    const mass = Mass!float(4);
    assert(vector * time == DVec3(8, -16, 24));
    assert(vector / time == DVec3(2, -4, 6));
    assert(vector / mass == DVec3(1, -2, 3));
    assert(8.0 / mass == 2.0);
    assert(8.0 * time == 16.0);
    assert(mass * 2.0 == 8.0);
    assert(vector - vector == zero!DVec3);
}

unittest
{
    static assert(!__traits(compiles, quantityFrom!float(double.init)));
    static assert(!__traits(compiles, quantityFrom!(Time!float)(double.init)));
    static assert(!__traits(compiles, quantityFrom!(Time!double)(float.init)));
    static assert(!__traits(compiles, float.init * Time!int(2)));
    static assert(!__traits(compiles, double.init / Mass!long(2)));
    assert(3.0 * Time!int(2) == 6.0);
}
