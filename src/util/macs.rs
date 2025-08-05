#[macro_export]
macro_rules! gxl_sh {
    ( $scope : expr ,$target : expr, $cmd : expr, $opt : expr , $exp : expr, $dict : expr) => {
        $crate::util::os_sh_realtime($scope, $target, $cmd, $opt, $exp, $dict)
    };
    ( $scope: expr ,$target : expr, $cmd : expr  ) => {
        $crate::utls::rg_sh(
            $scope,
            $target,
            $cmd,
            $crate::utls::ShellOpt::new(),
            $crate::eval::EnvExpress::from_env(),
            VarDict::default(),
        )
    };
}
