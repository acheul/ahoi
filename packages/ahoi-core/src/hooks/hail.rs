use super::*;

#[cfg_attr(debug_assertions, track_caller)]
fn help_set_read_hail<
    X: HailConverter<T> + 'static,
    T: 'static,
    Pipe: Pipeline<T> + 'static,
    const OPT: bool,
>(
    stock: ReadStock<T, Pipe, OPT>,
) -> X::HailValue {
    let parse_value = |value| X::__from_option_raw_value(value, OPT);

    let initial_hail_value = parse_value(stock.try_peek());

    let sphere_id = runtime::sphere::current_sphere_id().expect("set hail out of sphere");

    let _citer_id =
        runtime::insert::insert_hail_citer_runner_state((stock.value_id, stock.path), move || {
            let hail_value = parse_value(stock.try_read());
            runtime::propagation::mark_hail(sphere_id, hail_value);
        });

    // no initial run for hail citer
    return initial_hail_value;
}

#[cfg_attr(debug_assertions, track_caller)]
pub fn set_read_hail<
    X: HailConverter<T> + 'static,
    T: 'static,
    Pipe: Pipeline<T> + 'static,
    const OPT: bool,
>(
    stock: ReadStock<T, Pipe, OPT>,
) -> X::HailValue {
    let initial_hail_value = help_set_read_hail::<X, T, Pipe, OPT>(stock);

    // mark hail sphered
    runtime::sphere::register_hail_to_current_sphere(None);

    return initial_hail_value;
}

#[cfg_attr(debug_assertions, track_caller)]
pub fn set_hail<
    X: HailConverter<T> + 'static,
    T: 'static,
    Pipe: Pipeline<T> + Clone + 'static,
    const OPT: bool,
>(
    stock: Stock<T, Pipe, OPT>,
) -> X::HailValue {
    let initial_hail_value = help_set_read_hail::<X, T, Pipe, OPT>(stock.0.clone());

    // write callback
    let callback: Callback<X::HailValue, ()> = Callback::new(move |hail_value: X::HailValue| {
        let value = stock.try_write();
        let value = if OPT { value } else { Some(value.unwrap()) };
        if let Some(mut value) = value {
            let hail_value = X::into_raw_value(hail_value);
            *value = hail_value;
        }
    });

    // mark hail sphered
    runtime::sphere::register_hail_to_current_sphere(Some(callback.executer_id));

    return initial_hail_value;
}

pub fn write_hail<U: 'static>(sphere_id: SphereId, hail_value: U) -> Option<()> {
    let writer_id = runtime::sphere::get_hail_writer_id(sphere_id)?;
    runtime::run_runner::run_executer::<U, ()>(writer_id, hail_value)?;
    Some(())
}
