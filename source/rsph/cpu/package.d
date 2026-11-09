module rsph.cpu;

import rsph.config : Device;
import rsph.core.result : Result;
import rsph.core.traits : isModel, isParticle, isPhysicsDomain;

/// Raised by the unfinished CPU backend operations.
final class UnimplementedCpuOperation : Exception
{
    this(string operation)
    {
        super(operation ~ " is not implemented");
    }
}

struct CPUBuilder
{
    Result!(CPU, void) build()
    {
        throw new UnimplementedCpuOperation("CPUBuilder.build");
    }
}

struct CPU
{
    alias DeviceMarker = Device;

    struct RuntimeBuilder(D, M, P)
    if (isPhysicsDomain!D && isParticle!(D, P) && isModel!(D, P, M))
    {
        alias TargetDevice = CPU;
        alias Runtime = .Runtime!(D, M, P);
        alias Error = void;

        static Result!(Runtime, Error) build_runtime()
        {
            throw new UnimplementedCpuOperation("CPU.build_runtime");
        }
    }
}

struct Runtime(D, M, P)
if (isPhysicsDomain!D && isParticle!(D, P) && isModel!(D, P, M))
{
    private CPU device;
    private M model;

    private ref const(Scene) create_scene()
    {
        throw new UnimplementedCpuOperation("Runtime.create_scene");
    }
}

struct Scene {}
