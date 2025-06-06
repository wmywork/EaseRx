
#[macro_export]
macro_rules! combine_state_flow {
    // 入口点
    ($($signal:expr),+ $(,)?) => {
        combine_state_flow!(@process [] [] $($signal),+)
    };

    // 递归处理每个信号，构建变量名和绑定
    (@process [$($bindings:tt)*] [$($vars:ident)*] $signal:expr) => {
        // 最后一个信号，生成最终的 map_ref
        combine_state_flow!(@generate
            [$($bindings)* let signal_final = $signal,]
            [$($vars)* signal_final]
        )
    };

    (@process [$($bindings:tt)*] [$($vars:ident)*] $signal:expr, $($rest:expr),+) => {
        // 继续处理剩余信号
        combine_state_flow!(@process
            [$($bindings)* let signal_next = $signal,]
            [$($vars)* signal_next]
            $($rest),+
        )
    };

    // 生成最终的 map_ref 调用
    (@generate [$($bindings:tt)*] [$var:ident]) => {
        // 单个信号的情况
        map_ref! {
            $($bindings)*
            =>
            $var.clone()
        }
    };

    (@generate [$($bindings:tt)*] [$($vars:ident)+]) => {
        // 多个信号的情况
        map_ref! {
            $($bindings)*
            =>
            ($($vars.clone(),)+)
        }
    };
}