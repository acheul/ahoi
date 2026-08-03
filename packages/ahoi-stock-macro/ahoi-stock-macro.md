# ahoi-stock-macro

`#[derive(Stock)]` and `#[stock]` proc macros for the `ahoi` crate.

---

## Struct

### Named fields

```rust
#[derive(Stock)]
pub struct AB<T> {
    a: String,
    b: Vec<T>,
}
```

Generated:

```rust
pub const AB_A_KEY: u64 = 0u64;
pub const AB_B_KEY: u64 = 1u64;

pub trait ABStockExt<T, const __OPT: bool, __Pipe> {
    fn a(self) -> Stock<String, ChainedPipe<__Pipe, GetNext<AB<T>, String>, AB<T>, String>, __OPT>;
    fn b(self) -> Stock<Vec<T>, ChainedPipe<__Pipe, GetNext<AB<T>, Vec<T>>, AB<T>, Vec<T>>, __OPT>;
}

impl<T: 'static, const __OPT: bool, __Pipe> ABStockExt<T, __OPT, __Pipe>
    for Stock<AB<T>, __Pipe, __OPT>
{
    fn a(self) -> Stock<String, ChainedPipe<__Pipe, GetNext<AB<T>, String>, AB<T>, String>, __OPT> {
        self.derive(AB_A_KEY, GetNext::new(|x| &x.a, |x| &mut x.a))
    }

    fn b(self) -> Stock<Vec<T>, ChainedPipe<__Pipe, GetNext<AB<T>, Vec<T>>, AB<T>, Vec<T>>, __OPT> {
        self.derive(AB_B_KEY, GetNext::new(|x| &x.b, |x| &mut x.b))
    }
}
```

### Tuple struct — unnamed fields use `f{n}`

```rust
#[derive(Stock)]
pub struct CD(String, Vec<u32>);
```

Generated:

```rust
pub const CD_F0_KEY: u64 = 0u64;
pub const CD_F1_KEY: u64 = 1u64;

pub trait CDStockExt<const __OPT: bool, __Pipe> {
    fn f0(self) -> Stock<String, ChainedPipe<__Pipe, GetNext<CD, String>, CD, String>, __OPT>;
    fn f1(self)
    -> Stock<Vec<u32>, ChainedPipe<__Pipe, GetNext<CD, Vec<u32>>, CD, Vec<u32>>, __OPT>;
}

impl<const __OPT: bool, __Pipe> CDStockExt<__OPT, __Pipe> for Stock<CD, __Pipe, __OPT> {
    fn f0(self) -> Stock<String, ChainedPipe<__Pipe, GetNext<CD, String>, CD, String>, __OPT> {
        self.derive(CD_F0_KEY, GetNext::new(|x| &x.0, |x| &mut x.0))
    }
    fn f1(
        self,
    ) -> Stock<Vec<u32>, ChainedPipe<__Pipe, GetNext<CD, Vec<u32>>, CD, Vec<u32>>, __OPT> {
        self.derive(CD_F1_KEY, GetNext::new(|x| &x.1, |x| &mut x.1))
    }
}
```

---

## Enum

- **`{Name}StockExt`** — `{variant}()` for variants with exactly one field; returns an OPTIONAL `StockStruct<FieldType, ChainedPipe<…>, true>`.

Keys run `0..n`, one slot per variant regardless of whether an accessor is emitted.

```rust
#[derive(Stock)]
pub enum Shape<O> {
    Circle(O),               // circle()
    Rect { w: f64, h: f64 },
    Dot,
    Label(String),           // label()
}
```

Generated:

```rust
pub const SHAPE_CIRCLE_KEY: u64 = 0u64;
pub const SHAPE_LABEL_KEY: u64 = 3u64;

pub trait ShapeStockExt<O, __Pipe> {
    fn circle(self) -> Stock<O, ChainedPipe<__Pipe, GetNextOpt<Shape<O>, O>, Shape<O>, O>, true>;
    fn label(
        self,
    ) -> Stock<String, ChainedPipe<__Pipe, GetNextOpt<Shape<O>, String>, Shape<O>, String>, true>;
}

impl<O: 'static, const __OPT: bool, __Pipe> ShapeStockExt<O, __Pipe>
    for Stock<Shape<O>, __Pipe, __OPT>
{
    fn circle(self) -> Stock<O, ChainedPipe<__Pipe, GetNextOpt<Shape<O>, O>, Shape<O>, O>, true> {
        self.try_derive(
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
    ) -> Stock<String, ChainedPipe<__Pipe, GetNextOpt<Shape<O>, String>, Shape<O>, String>, true>
    {
        self.try_derive(
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
```

---

## Skipping fields / variants

`#[stock(skip)]` suppresses const and method generation. The index slot is still
consumed, keeping subsequent keys stable.

```rust
#[derive(Stock)]
struct Tagged {
    name: String,
    #[stock(skip)]  // no TAGGED_META_KEY or .meta() generated
    _meta: u64,
    value: u32,     // gets key 2
}

// TAGGED_NAME_KEY  = 0
// TAGGED_VALUE_KEY = 2  (gap at 1 for _meta)
```

---

## CamelCase → snake_case

Variant and field names are converted to snake_case for method names.
`TriAngle` → `is_tri_angle()` / `tri_angle()`.

---

# `#[stock]` macro

```rust
#[derive(Stock)]
struct AB<T> {
    a: String,
    b: Vec<T>,
}


#[stock]
impl<T: 'static, Pipe: Pipeline<AB<T>> + Clone + 'static> Stock<AB<T>, Pipe> {
    fn update_a(&self, new_a: &str) -> usize {
        let mut a = self.clone().a().read_mut();
        a.push_str(new_a);
        return a.len();
    }

    fn length_of_b<A>(self, _any: A) -> Option<T> {
        self.b().read_mut().pop()
    }
}
```

- Above code should be expanded into something like this:

```rust
pub trait ABStockExt2<T: 'static> {
    fn update_a(&self, new_a: &str) -> usize;

    fn length_of_b<A>(self, _any: A) -> Option<T>;
}

impl<T: 'static, Pipe: Pipeline<AB<T>> + Clone + 'static> ABStockExt2<T> for Stock<AB<T>, Pipe> {
    fn update_a(&self, new_a: &str) -> usize {
        let mut a = self.clone().a().read_mut();
        a.push_str(new_a);
        return a.len();
    }

    fn length_of_b<A>(self, _any: A) -> Option<T> {
        self.b().read_mut().pop()
    }
}
```

- More details
  - `#[stock(ABStockExtCustom)]`: trait 이름 설정 가능. 설정 안 할 경우 {type}StockExt2
