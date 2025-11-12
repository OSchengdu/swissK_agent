pub fn eval_expr(expr: &str) -> Result<f64, Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    let mut vars = HashMap::new();
    let result = meval::eval_str_with_context(expr, &mut vars)?;
    Ok(result)
}
