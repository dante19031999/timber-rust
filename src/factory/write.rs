use crate::service::Vector;

pub struct WriteFactory;

pub struct VectorFactory {
    capacity: usize,
}

impl WriteFactory {
    pub fn io() {}

    pub fn fmt() {}

    pub fn vector() -> VectorFactory {
        VectorFactory::default()
    }

    pub fn vector_with_capacity(capacity: usize) -> VectorFactory {
        VectorFactory { capacity }
    }
}

impl VectorFactory {
    pub fn new(capacity: usize) -> Self {
        Self { capacity }
    }

    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub fn capacity(self, capacity: usize) -> Self {
        Self { capacity, ..self }
    }

    pub fn build_service(self) -> Box<Vector> {
        Vector::new(self.capacity)
    }
}

impl Default for VectorFactory {
    fn default() -> Self {
        Self { capacity: 1024 }
    }
}
