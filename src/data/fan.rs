use measurements::AngularVelocity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanData {
    /// The position or index of the fan as seen by the device
    /// Usually dependent on where to fan is connected to the control board
    pub position: i16,
    /// The RPM of the fan
    pub rpm: AngularVelocity,
}
