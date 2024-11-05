use game::Game;

pub trait Policy {
    fn eval(game: Game) -> ([f32; 81], f32);
}

pub struct UltimateXONNPolicy {
    linear_1: nn::linear,
    linear_2: nn::linear,
    linear_3: nn::linear,
}

impl UltimateXONNPolicy {
    fn new()
}

impl Policy for UltimateXONNPolicy {
    fn eval(game: Game) -> ([f32, 81], f32) {

    }
}