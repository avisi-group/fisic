use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::eyre::{eyre, Result};
use fisic::{
    actions::Action,
    actions::{
        create::{invoke as InvokeCreate, CreateActionArgs},
        init::{InitActionArgs, PartitionTableType},
    },
    image::Image,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct FisicArgs {
    #[command(subcommand)]
    action: ActionCommand,

    #[arg(short, required = true)]
    image: String,
}

#[derive(Args, Debug)]
struct CreateAction {
    #[arg(long)]
    size: String,

    #[arg(long, action)]
    overwrite: bool,

    #[arg(long)]
    init_pt: Option<InitType>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum InitType {
    None,
    MBR,
    GPT,
}

#[derive(Args, Debug)]
struct InitAction {
    #[arg(long)]
    init_type: InitType,
}

#[derive(Subcommand, Debug)]
enum ActionCommand {
    Create(CreateAction),
    Init(InitAction),
    Info,
    List,
}

impl TryFrom<CreateAction> for CreateActionArgs {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: CreateAction) -> Result<Self, Self::Error> {
        Ok(CreateActionArgs {
            overwrite: value.overwrite,
            initial_pt_type: match value.init_pt {
                Some(InitType::MBR) => PartitionTableType::MBR,
                Some(InitType::GPT) => PartitionTableType::GPT,
                _ => PartitionTableType::None,
            },
            size: parse_size::parse_size(value.size)
                .map_err(|e| eyre!("size parsing failed: {}", e))?
                .try_into()?,
        })
    }
}

impl TryFrom<InitAction> for InitActionArgs {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: InitAction) -> Result<Self, Self::Error> {
        Ok(InitActionArgs {
            pt_type: match value.init_type {
                InitType::None => PartitionTableType::None,
                InitType::MBR => PartitionTableType::MBR,
                InitType::GPT => PartitionTableType::GPT,
            },
        })
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = FisicArgs::parse();

    Ok(match args.action {
        ActionCommand::Create(a) => InvokeCreate(&args.image, a.try_into()?)?,
        _ => {
            let mut image = Image::open(args.image)?;

            match args.action {
                ActionCommand::Init(a) => {
                    fisic::actions::init::InitAction::invoke(&mut image, a.try_into()?)?
                }
                _ => panic!("unsupported"),
            }
        }
    })
}
