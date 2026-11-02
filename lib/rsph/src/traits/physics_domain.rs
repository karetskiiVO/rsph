pub trait PhysicsDomain {
    type Time;

    type Dencity;
    type Force;

    type Position;
    type Velocity;
    type Acceleration;
}
