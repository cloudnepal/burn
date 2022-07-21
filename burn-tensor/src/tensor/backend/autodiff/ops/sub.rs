use crate::{
    backend::autodiff::{ADFloat, ADFloatTensor, ADTensor},
    define_ops, execute_ops,
    ops::{
        BinaryOps, BinaryOpsNodeState, BinaryRecordedOps, UnaryOps, UnaryOpsNodeState,
        UnaryRecordedOps,
    },
    register_ops, TensorOpsSub,
};
use num_traits::Float;

register_ops!(
    ops BinaryOps<T, T, T>,
    name ADTensorSubOps,
    partial_left |state: &BinaryOpsNodeState<T, T, T>| {
        state.output.borrow_mut().grad() * state.left.borrow().value().ones()
    },
    partial_right |state: &BinaryOpsNodeState<T, T, T>| {
        state.output.borrow_mut().grad() * state.right.borrow().value().ones().neg()
    },
);

register_ops!(
    ops UnaryOps<T, T>,
    name ADTensorSubScalarOps state P,
    partial |_state, state_recorded: &UnaryOpsNodeState<T, T>|{
        state_recorded.output.borrow_mut().grad() * state_recorded.input.borrow().value().ones()
    },
);

impl<T, P, const D: usize> TensorOpsSub<P, D> for ADTensor<P, D, T>
where
    T: ADFloatTensor<P, D>,
    P: ADFloat,
{
    fn sub(&self, other: &Self) -> Self {
        let node = execute_ops!(
            lhs self.node.clone(),
            rhs other.node.clone(),
            out TensorOpsSub::sub(&self.tensor(), &other.tensor()),
            ops ADTensorSubOps::new(),
        );
        self.from_existing(node)
    }

    fn sub_scalar(&self, other: &P) -> Self {
        let node = execute_ops!(
            input self.node.clone(),
            out TensorOpsSub::sub_scalar(&self.tensor(), &other),
            ops ADTensorSubScalarOps::new(other.clone()),
        );
        self.from_existing(node)
    }
}

impl<T, P, const D: usize> std::ops::Sub<P> for ADTensor<P, D, T>
where
    T: ADFloatTensor<P, D> + 'static,
    P: ADFloat + 'static,
{
    type Output = ADTensor<P, D, T>;

    fn sub(self, rhs: P) -> Self::Output {
        TensorOpsSub::sub_scalar(&self, &rhs)
    }
}

impl<T, P, const D: usize> std::ops::Sub<ADTensor<P, D, T>> for ADTensor<P, D, T>
where
    T: ADFloatTensor<P, D> + 'static,
    P: ADFloat + 'static,
{
    type Output = ADTensor<P, D, T>;

    fn sub(self, rhs: Self) -> Self::Output {
        TensorOpsSub::sub(&self, &rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{backend::autodiff::helper::ADTchTensor, Data, TensorBase, TensorOpsSub};

    #[test]
    fn should_diff_sub() {
        let data_1 = Data::from([2.0, 5.0]);
        let data_2 = Data::from([4.0, 1.0]);

        let tensor_1 = ADTchTensor::from_data(data_1.clone());
        let tensor_2 = ADTchTensor::from_data(data_2.clone());

        let tensor_3 = tensor_1.clone() - tensor_2.clone();
        tensor_3.backward();

        let grad_1 = tensor_1.grad();
        let grad_2 = tensor_2.grad();

        assert_eq!(grad_1.into_data(), Data::from([1.0, 1.0]));
        assert_eq!(grad_2.into_data(), Data::from([-1.0, -1.0]));
        assert_eq!(tensor_3.into_data(), Data::from([-2.0, 4.0]));
    }

    #[test]
    fn should_diff_sub_scalar() {
        let data = Data::from([2.0, 10.0]);
        let tensor = ADTchTensor::from_data(data.clone());
        let tensor_out = tensor.clone() - 5.0;
        tensor_out.backward();

        let grad = tensor.grad();
        assert_eq!(grad.into_data(), Data::from([1.0, 1.0]));
        assert_eq!(tensor_out.into_data(), Data::from([-3.0, 5.0]));
    }

    #[test]
    fn test_sub_complex_1() {
        let data_1: Data<f64, 2> = Data::from([[1.0, 7.0], [13.0, -3.0]]);
        let data_2: Data<f64, 2> = Data::from([[4.0, 7.0], [2.0, 3.0]]);
        let data_3: Data<f64, 2> = Data::from([[2.0, 2.0], [2.0, 2.0]]);

        let tensor_1 = ADTchTensor::from_data(data_1.clone());
        let tensor_2 = ADTchTensor::from_data(data_2.clone());
        let tensor_3 = ADTchTensor::from_data(data_3.clone());

        let tensor_4 = tensor_1.sub(&tensor_2);
        let tensor_5 = tensor_4.sub(&tensor_3).sub_scalar(&5.0);
        let tensor_6 = tensor_1.sub(&tensor_5);

        tensor_6.backward();

        let grad_1 = tensor_1.grad();
        let grad_2 = tensor_2.grad();

        assert_eq!(grad_1.into_data(), Data::from([[0.0, 0.0], [0.0, 0.0]]));
        assert_eq!(grad_2.into_data(), Data::from([[1.0, 1.0], [1.0, 1.0]]));
    }
}