use measurements::AngularVelocity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanData {
    pub position: i16,
    pub rpm: AngularVelocity,
}
