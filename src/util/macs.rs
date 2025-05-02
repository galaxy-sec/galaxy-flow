#[macro_export]
macro_rules! rg_sh {
    ( $scope : expr ,$target : expr, $cmd : expr, $opt : expr , $exp : expr) => {
        $crate::util::gxl_sh($scope, $target, $cmd, $opt, $exp)
    };
    ( $scope: expr ,$target : expr, $cmd : expr  ) => {
        $crate::utl::gxl_sh(
            $scope,
            $target,
            $cmd,
            $crate::utls::ShellOpt::new(),
            $crate::eval::EnvExpress::from_env(),
        )
    };
}
