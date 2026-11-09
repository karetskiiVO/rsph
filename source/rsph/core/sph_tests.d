module rsph.core.sph_tests;

version (unittest)
{
    import rsph.core.quantities : DVec3, Mass, Time;
    import rsph.core.sph : TimeStepConfig, TimeStepState, calc_dt, calc_internal_acceleration,
        calc_presure, calc_viscosity_acceleration, update_density;

    private struct TestDomain(Scalar, Duration = Scalar, MassValue = Scalar, DensityValue = Scalar)
    {
        alias Time = Duration;
        alias Length = Scalar;
        alias Mass = MassValue;
        alias Density = DensityValue;
        alias Force = DVec3;
        alias Position = DVec3;
        alias Displacement = DVec3;
        alias Velocity = DVec3;
        alias Acceleration = DVec3;
        alias Pressure = Scalar;
        alias SpecificInternalEnergy = Scalar;
    }

    private alias F64Domain = TestDomain!double;
    private alias WrappedDomain = TestDomain!(float, Time!float);
    private alias WrappedF64Domain = TestDomain!(double, Time!double);

    private TimeStepConfig!double config()
    {
        return TimeStepConfig!double(0.25, 0.25, 0.125, 1.0);
    }

    private TimeStepState!double resting()
    {
        return TimeStepState!double(0.0, 0.0, 0.0, 0.0);
    }

    @("acoustic limit uses sound and flow speed")
    unittest
    {
        auto state = resting();
        state.max_speed = 1.0;
        state.max_sound_speed = 3.0;
        assert(calc_dt!F64Domain(2.0, state, config()) == 0.125);
    }

    @("acceleration and viscosity each limit the step")
    unittest
    {
        auto acceleration = resting();
        acceleration.max_acceleration = 4.0;
        assert(calc_dt!F64Domain(4.0, acceleration, config()) == 0.25);

        auto viscosity = resting();
        viscosity.max_kinematic_viscosity = 4.0;
        assert(calc_dt!F64Domain(2.0, viscosity, config()) == 0.125);
    }

    @("selects the strictest limit and respects the cap")
    unittest
    {
        const state = TimeStepState!double(1.0, 1.0, 16.0, 2.0);
        assert(calc_dt!F64Domain(2.0, state, config()) == 0.08838834764831845);

        auto capped = config();
        capped.max_dt = 0.01;
        assert(calc_dt!F64Domain(2.0, state, capped) == 0.01);
    }

    @("zero rates use the cap and tiny positive rates still limit the step")
    unittest
    {
        assert(calc_dt!F64Domain(2.0, resting(), config()) == 1.0);

        auto tiny = resting();
        tiny.max_sound_speed = 1e-20;
        auto uncapped = config();
        uncapped.max_dt = 1e30;
        assert(calc_dt!F64Domain(2.0, tiny, uncapped) == 5e19);
    }

    @("supports single precision with a wrapped time")
    unittest
    {
        const state = TimeStepState!float(1.0f, 3.0f, 0.0f, 0.0f);
        const configured = TimeStepConfig!(float, Time!float)(0.5f, 0.25f, 0.125f, Time!float(1.0f));
        Time!float step = calc_dt!WrappedDomain(2.0f, state, configured);
        assert(step == Time!float(0.25f));
    }

    @("returns the wrapped time cap")
    unittest
    {
        const configured = TimeStepConfig!(double, Time!double)(0.25, 0.25, 0.125, Time!double(0.01));
        Time!double step = calc_dt!WrappedF64Domain(2.0, resting(), configured);
        assert(step == Time!double(0.01));
    }

    private alias ParticleDomain = TestDomain!(double, double, Mass!double);

    private struct TestParticle(Density = double)
    {
        DVec3 location;
        DVec3 speed;
        DVec3 acceleration_value;
        Mass!double mass;
        Density rest_density;
        Density density_value;
        double pressure_value;
        double energy;

        DVec3 position() const { return location; }
        DVec3 velocity() const { return speed; }
        DVec3 acceleration() const { return acceleration_value; }
        Mass!double rest_mass() const { return mass; }
        Density rest_mass_density() const { return rest_density; }
        Density density() const { return density_value; }
        double pressure() const { return pressure_value; }
        double specific_internal_energy() const { return energy; }

        void set_position(DVec3 value) { location = value; }
        void set_velocity(DVec3 value) { speed = value; }
        void set_acceleration(DVec3 value) { acceleration_value = value; }
        void set_density(Density value) { density_value = value; }
        void set_pressure(double value) { pressure_value = value; }
        void set_specific_internal_energy(double value) { energy = value; }
    }

    private struct TestKernel
    {
        alias Value = double;
        alias Gradient = DVec3;
        alias Laplacian = double;

        double support_radius(double h) const { return h; }

        Value value(DVec3 r, double h) const
        {
            import std.algorithm.comparison : max;

            return max(0.0, h * h - (r.x * r.x + r.y * r.y + r.z * r.z));
        }

        Gradient gradient(DVec3 r, double h) const
        {
            return value(r, h) > 0.0 ? r * -2.0 : DVec3.ZERO;
        }

        Laplacian viscosity_laplacian(DVec3 r, double h) const
        {
            return value(r, h);
        }
    }

    private struct TestEquationOfState
    {
        double pressure(double rest_density, double energy) const
        {
            return rest_density * (energy + 1.0);
        }
    }

    private struct TestModel
    {
        alias Kernel = TestKernel;
        alias EquationOfState = TestEquationOfState;

        private Kernel kernel_value;
        private EquationOfState eos;

        ref const(Kernel) kernel() const { return kernel_value; }
        ref const(EquationOfState) equation_of_state() const { return eos; }
        double h() const { return 2.0; }
    }

    @("accumulates neighbour density and forces and evaluates pressure in the rest frame")
    unittest
    {
        auto particle = TestParticle!double(DVec3(1, 0, 0), DVec3(3, 0, 0), DVec3(0.5, 0, 0),
            Mass!double(1), 2.0, 2.0, 0.0, 5.0);
        const neighbour = TestParticle!double(DVec3.ZERO, DVec3(1, 0, 0), DVec3.ZERO,
            Mass!double(2), 4.0, 4.0, 5.0, 0.0);
        const TestModel model;

        update_density!ParticleDomain(model, particle, neighbour);
        assert(particle.density() == 8.0);

        calc_presure!ParticleDomain(model, particle);
        assert(particle.pressure() == 12.0);

        calc_internal_acceleration!ParticleDomain(model, particle, neighbour);
        assert(particle.acceleration() == DVec3(2.5, 0, 0));

        calc_viscosity_acceleration!ParticleDomain(model, particle, neighbour, 8.0);
        assert(particle.acceleration() == DVec3(-0.5, 0, 0));
    }

    @("rejects a mass-weighted kernel value with a wider type than density")
    unittest
    {
        import rsph.core.traits : isModel, isSphParticle;

        alias NarrowDomain = TestDomain!(double, double, Mass!double, float);
        static assert(isSphParticle!(NarrowDomain, TestParticle!float));
        static assert(isModel!(NarrowDomain, TestParticle!float, TestModel));

        TestParticle!float particle, neighbour;
        const TestModel model;
        static assert(!__traits(compiles, update_density!NarrowDomain(model, particle, neighbour)));
    }
}
