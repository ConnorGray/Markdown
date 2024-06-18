use wolfram_library_link::expr::{Expr, Symbol};

pub(crate) fn try_headed(e: &Expr, head: Symbol) -> Result<&[Expr], String> {
    let e = match e.try_as_normal() {
        Some(value) => value,
        None => return Err(format!("expected {}[..]", head.symbol_name())),
    };

    if !e.has_head(&head) {
        return Err(format!("expected {}[..]", head.symbol_name()));
    }

    Ok(e.elements())
}

#[allow(unused)]
pub(crate) fn try_headed_len<const LEN: usize>(
    e: &Expr,
    head: Symbol,
) -> Result<&[Expr; LEN], String> {
    let elems = try_headed(e, head.clone())?;

    let Ok(elems): Result<&[Expr; LEN], _> = elems.try_into() else {
        return Err(format!(
            "expected {}[..] with length {LEN}",
            head.symbol_name()
        ));
    };

    Ok(elems)
}
