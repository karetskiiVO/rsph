module rsph.config;

import rsph.core.result : Result;
import rsph.core.traits : isModel, isParticle, isPhysicsDomain;

/// Marker for devices. Opt in with `alias DeviceMarker = Device`.
struct Device {}

enum bool isDevice(Dev) = is(Dev.DeviceMarker == Device);

/// A builder selects a device and exposes a static typed runtime factory.
enum bool isBuildRuntime(Dev, Builder) = isDevice!Dev
    && is(Builder.TargetDevice == Dev)
    && is(Builder.Runtime) && is(Builder.Error)
    && __traits(compiles, { Builder.build_runtime(); })
    && is(typeof(Builder.build_runtime()) == Result!(Builder.Runtime, Builder.Error));

/** Selects the execution device and physical model.
 * A device can supply a `RuntimeBuilder!(D, M, P)` that satisfies `isBuildRuntime`
 * to make its factory and result types available on the configuration.
 */
struct Config(D, Dev, M, P)
if (isPhysicsDomain!D && isDevice!Dev && isParticle!(D, P) && isModel!(D, P, M))
{
    Dev device;
    M model;

    static if (is(Dev.RuntimeBuilder!(D, M, P)))
    {
        private alias Builder = Dev.RuntimeBuilder!(D, M, P);

        static if (isBuildRuntime!(Dev, Builder))
        {
            alias TargetDevice = Dev;
            alias Runtime = Builder.Runtime;
            alias Error = Builder.Error;
            alias build_runtime = Builder.build_runtime;
        }
    }
}
