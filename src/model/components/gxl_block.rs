use super::prelude::*;

use crate::ability::delegate::ActCall;
use crate::ability::GxAssert;
use crate::ability::GxCmd;
use crate::ability::GxEcho;
use crate::ability::GxRead;
use crate::ability::GxTpl;
use crate::ability::RgVersion;
use crate::context::ExecContext;

use crate::calculate::cond::CondExec;
use crate::model::components::gxl_cond::RunArgs;

use super::gxl_cond::RgCond;
use super::gxl_spc::GxlSpace;
use super::gxl_var::RgProp;

#[derive(Clone, Debug)]
pub enum BlockAction {
    Command(GxCmd),
    Cond(RgCond),
    Echo(GxEcho),
    Assert(GxAssert),
    Version(RgVersion),
    Read(GxRead),
    Delegate(ActCall),
    Tpl(GxTpl),
    //Vault(GxVault),
}

#[derive(Clone, Getters, Default, Debug)]
pub struct BlockNode {
    props: Vec<RgProp>,
    items: Vec<BlockAction>,
}

impl BlockNode {
    pub fn new() -> Self {
        Self {
            props: vec![],
            items: vec![],
        }
    }
}

impl CondExec for BlockNode {
    fn cond_exec(&self, args: &mut RunArgs) -> EOResult {
        self.exec(args.ctx.clone(), &mut args.def.clone())
    }
}
impl RunnableTrait for BlockAction {
    fn exec(&self, ctx: ExecContext, dct: &mut VarsDict) -> EOResult {
        match self {
            BlockAction::Command(o) => o.exec(ctx, dct),
            BlockAction::Echo(o) => o.exec(ctx, dct),
            BlockAction::Assert(o) => o.exec(ctx, dct),
            BlockAction::Cond(o) => o.exec(ctx, dct),
            BlockAction::Read(o) => o.exec(ctx, dct),
            BlockAction::Version(o) => o.exec(ctx, dct),
            BlockAction::Tpl(o) => o.exec(ctx, dct),
            BlockAction::Delegate(o) => o.exec(ctx, dct),
            //BlockAction::Vault(o) => o.exec(ctx, dct),
        }
    }
}
impl RunnableTrait for BlockNode {
    fn exec(&self, ctx: ExecContext, var_dict: &mut VarsDict) -> EOResult {
        //ctx.append("block");
        let mut job = Job::from("block");
        self.export_props(ctx.clone(), var_dict, "")?;

        for item in &self.items {
            let task = item.exec(ctx.clone(), var_dict)?;
            job.append(task);
        }
        Ok(ExecOut::Job(job))
    }
}
impl DependTrait<&GxlSpace> for BlockNode {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        //let mut ins = BlockNode::default();
        //ins.props = self.props().clone();
        let mut ins = BlockNode {
            props: self.props().clone(),
            ..Default::default()
        };
        for x in self.items {
            let item = match x {
                BlockAction::Tpl(v) => BlockAction::Tpl(v.clone()),
                BlockAction::Cond(v) => BlockAction::Cond(v.clone()),
                BlockAction::Read(v) => BlockAction::Read(v.clone()),
                BlockAction::Echo(v) => BlockAction::Echo(v.clone()),
                //BlockAction::Vault(v) => BlockAction::Vault(v.clone()),
                BlockAction::Assert(v) => BlockAction::Assert(v.clone()),
                BlockAction::Version(v) => BlockAction::Version(v.clone()),
                BlockAction::Command(v) => BlockAction::Command(v.clone()),
                BlockAction::Delegate(v) => BlockAction::Delegate(v.assemble(mod_name, src)?),
            };
            ins.append(item);
        }
        Ok(ins)
    }
}
impl PropsTrait for BlockNode {
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

impl AppendAble<RgProp> for BlockNode {
    fn append(&mut self, prop: RgProp) {
        self.props.push(prop);
    }
}
impl AppendAble<BlockAction> for BlockNode {
    fn append(&mut self, hold: BlockAction) {
        self.items.push(hold);
    }
}
impl AppendAble<Vec<BlockAction>> for BlockNode {
    fn append(&mut self, holds: Vec<BlockAction>) {
        for h in holds {
            self.items.push(h);
        }
    }
}

impl AppendAble<SequEnum> for BlockNode {
    fn append(&mut self, _v: SequEnum) {
        todo!();
    }
}

#[cfg(test)]
mod tests {

    use orion_common::friendly::New2;

    //test RgBlock append
    use super::*;
    use crate::model::components::gxl_block::BlockNode;
    #[test]
    fn test_append() {
        let mut block = BlockNode::new();
        let prop = RgProp::new("test", "hello");
        block.append(prop);
        assert_eq!(block.props.len(), 1);
    }
    //test RgBlock exec method
    #[test]
    fn test_exec() {
        let mut block = BlockNode::new();
        let prop = RgProp::new("test", "hello");
        block.append(prop);
        let ctx = ExecContext::new(false);
        let mut def = VarsDict::default();
        let res = block.exec(ctx, &mut def);
        assert_eq!(res.is_ok(), true);
    }

    /*
    #[test]
    fn test_exec2() {
        use crate::exec::exec_init_env;
        use crate::stc::rg_cond::RgCond;
        let (ctx, mut def) = exec_init_env();
        let mut block = RgBlock::new();
        block.append(RgProp::new("key1", "val1"));
        let ctrl_express: IFExpress = IFExpressT::if_ins(
            ExpressEnum::MU32(BinExpress::equal_ins(MocU32::from("moc_1"), 1)),
            Arc::new(RgBlock::new()),
            Arc::new(RgBlock::new()),
        );
        let (_tx, rx) = channel();
        let (tx1, _rx1) = channel();

        block.append(Arc::new(RgCond {
            cond: Arc::new(ctrl_express),
        }) as RunHold);
        block.exec(ctx.clone(), &mut def, (rx, tx1)).unwrap();
    }
    */
}
