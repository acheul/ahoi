//! Prototypes to illustrate `#[derive(Stock)]` & `[stock]` extensions
use super::*;

/**
 * Struct, Named fields
```rust, no_run
#[derive(Stock)]
pub struct AB<T> {
    a: String,
    b: Vec<T>,
}
```
 */
pub struct AB<T> {
    a: String,
    b: Vec<T>,
}

pub const AB_A_KEY: u64 = 0u64;
pub const AB_B_KEY: u64 = 1u64;

pub trait ABStockExt<T, __Pipe>: Derivable<AB<T>, __Pipe> {
    fn a(
        self,
    ) -> <Self as Derivable<AB<T>, __Pipe>>::DeriveType<
        String,
        ChainedPipe<__Pipe, GetNext<AB<T>, String>, AB<T>, String>,
    >;
    fn b(
        self,
    ) -> <Self as Derivable<AB<T>, __Pipe>>::DeriveType<
        Vec<T>,
        ChainedPipe<__Pipe, GetNext<AB<T>, Vec<T>>, AB<T>, Vec<T>>,
    >;
}

impl<__Target, T: 'static, __Pipe> ABStockExt<T, __Pipe> for __Target
where
    __Target: Derivable<AB<T>, __Pipe>,
{
    fn a(
        self,
    ) -> <Self as Derivable<AB<T>, __Pipe>>::DeriveType<
        String,
        ChainedPipe<__Pipe, GetNext<AB<T>, String>, AB<T>, String>,
    > {
        self.derive(AB_A_KEY, GetNext::new(|x| &x.a, |x| &mut x.a))
    }

    fn b(
        self,
    ) -> <Self as Derivable<AB<T>, __Pipe>>::DeriveType<
        Vec<T>,
        ChainedPipe<__Pipe, GetNext<AB<T>, Vec<T>>, AB<T>, Vec<T>>,
    > {
        self.derive(AB_B_KEY, GetNext::new(|x| &x.b, |x| &mut x.b))
    }
}

#[test]
fn run_example_of_manual_extension() {
    crate::make_sphere(None, || {
        let ab = Stock::new(AB {
            a: "A".to_string(),
            b: vec![1, 2],
        });
        crate::batch(|| {
            let _ = ab.update_a(&"hey");
            let _ = ab.length_of_b(());
        });
        let _a = ab.a();

        let abs = Stock::new(vec![AB {
            a: "A".to_string(),
            b: vec![1, 2],
        }]);
        let x = abs.get(0);
        let _a = x.a();

        // read-only stocks derive too: ReadStock -> ReadStock, and an optional
        // step (`get`) collapses to OptReadStock.
        let ro: &ReadStock<AB<i32>> = &ab;
        let _ro_a = ro.clone().a();
    });
}

/**
 * Struct, Unnamed fields
```rust, no_run
#[derive(Stock)]
pub struct CD(String, Vec<u32>);
```
 */
pub struct CD(String, Vec<u32>);

pub const CD_F0_KEY: u64 = 0u64;
pub const CD_F1_KEY: u64 = 1u64;

pub trait CDStockExt<__Pipe>: Derivable<CD, __Pipe> {
    fn f0(
        self,
    ) -> <Self as Derivable<CD, __Pipe>>::DeriveType<
        String,
        ChainedPipe<__Pipe, GetNext<CD, String>, CD, String>,
    >;
    fn f1(
        self,
    ) -> <Self as Derivable<CD, __Pipe>>::DeriveType<
        Vec<u32>,
        ChainedPipe<__Pipe, GetNext<CD, Vec<u32>>, CD, Vec<u32>>,
    >;
}

impl<__Target, __Pipe> CDStockExt<__Pipe> for __Target
where
    __Target: Derivable<CD, __Pipe>,
{
    fn f0(
        self,
    ) -> <Self as Derivable<CD, __Pipe>>::DeriveType<
        String,
        ChainedPipe<__Pipe, GetNext<CD, String>, CD, String>,
    > {
        self.derive(CD_F0_KEY, GetNext::new(|x| &x.0, |x| &mut x.0))
    }
    fn f1(
        self,
    ) -> <Self as Derivable<CD, __Pipe>>::DeriveType<
        Vec<u32>,
        ChainedPipe<__Pipe, GetNext<CD, Vec<u32>>, CD, Vec<u32>>,
    > {
        self.derive(CD_F1_KEY, GetNext::new(|x| &x.1, |x| &mut x.1))
    }
}

/**
 * Enum
```rust, no_run
#[derive(Stock)]
pub enum Shape<O> {
    Circle(O),               // circle()
    Rect { w: f64, h: f64 },
    Dot,
    Label(String),           // label()
}
```
 */
pub enum Shape<O> {
    Circle(O), // circle()
    Rect { w: f64, h: f64 },
    Dot,
    Label(String), // label()
}

pub const SHAPE_CIRCLE_KEY: u64 = 0u64;
pub const SHAPE_LABEL_KEY: u64 = 3u64;

pub trait ShapeStockExt<O, __Pipe>: Derivable<Shape<O>, __Pipe> {
    fn circle(
        self,
    ) -> <Self as Derivable<Shape<O>, __Pipe>>::DeriveOptType<
        O,
        ChainedPipe<__Pipe, GetNextOpt<Shape<O>, O>, Shape<O>, O>,
    >;
    fn label(
        self,
    ) -> <Self as Derivable<Shape<O>, __Pipe>>::DeriveOptType<
        String,
        ChainedPipe<__Pipe, GetNextOpt<Shape<O>, String>, Shape<O>, String>,
    >;
}

impl<__Target, O: 'static, __Pipe> ShapeStockExt<O, __Pipe> for __Target
where
    __Target: Derivable<Shape<O>, __Pipe>,
{
    fn circle(
        self,
    ) -> <Self as Derivable<Shape<O>, __Pipe>>::DeriveOptType<
        O,
        ChainedPipe<__Pipe, GetNextOpt<Shape<O>, O>, Shape<O>, O>,
    > {
        self.derive_opt(
            SHAPE_CIRCLE_KEY,
            GetNextOpt::new(
                |x| {
                    if let Shape::Circle(v) = x {
                        Some(v)
                    } else {
                        None
                    }
                },
                |x| {
                    if let Shape::Circle(v) = x {
                        Some(v)
                    } else {
                        None
                    }
                },
            ),
        )
    }
    fn label(
        self,
    ) -> <Self as Derivable<Shape<O>, __Pipe>>::DeriveOptType<
        String,
        ChainedPipe<__Pipe, GetNextOpt<Shape<O>, String>, Shape<O>, String>,
    > {
        self.derive_opt(
            SHAPE_LABEL_KEY,
            GetNextOpt::new(
                |x| {
                    if let Shape::Label(v) = x {
                        Some(v)
                    } else {
                        None
                    }
                },
                |x| {
                    if let Shape::Label(v) = x {
                        Some(v)
                    } else {
                        None
                    }
                },
            ),
        )
    }
}

/**
 * # `#[stock]` extension
```rust, no_run
#[stock]
impl<T: 'static, Pipe: Pipeline<AB<T>> + Clone + 'static> Stock<AB<T>, Pipe> {
    fn update_a(&self, new_a: &str) -> usize {
        let mut a = self.clone().a().write();
        a.push_str(new_a);
        return a.len();
    }

    fn length_of_b<A>(self, _any: A) -> Option<T> {
        self.b().write().pop()
    }
}
```
 */
pub trait ABStockExt2<T: 'static> {
    fn update_a(&self, new_a: &str) -> usize;

    fn length_of_b<A>(self, _any: A) -> Option<T>;
}

impl<T: 'static, Pipe: Pipeline<AB<T>> + Clone + 'static> ABStockExt2<T> for Stock<AB<T>, Pipe> {
    fn update_a(&self, new_a: &str) -> usize {
        let mut a = self.clone().a().write();
        a.push_str(new_a);
        return a.len();
    }

    fn length_of_b<A>(self, _any: A) -> Option<T> {
        self.b().write().pop()
    }
}
