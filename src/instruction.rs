//! Instruction types


use std::mem::size_of;
use crate::{
    error::LendingError,
};
use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::PUBKEY_BYTES;
use crate::util::unpack_util::{unpack_bool, unpack_u16};
use crate::util::unpack_util::{
    unpack_bytes32,
    unpack_pubkey,
    unpack_u64,
    unpack_u8
};

/// Instructions supported by the lending program.
#[derive(Clone, Debug, PartialEq)]
pub enum LendingInstruction {
    // 0
    /// Initializes a new lending market.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[singer]` Init lending market authority
    ///   1. `[writable]` Lending market account - uninitialized.
    ///   2. `[]` Rent sysvar.
    ///   3. `[]` Token program id.
    ///   4. `[]` Pyth oracle program id.
    ///   5. `[]` Larix oracle program id.
    ///   6. `[]` Larix oracle id.
    InitLendingMarket {
        /// Owner authority which can add new reserves
        owner: Pubkey,
        /// Currency market prices are quoted in
        /// e.g. "USD" null padded (`*b"USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"`) or a SPL token mint pubkey
        quote_currency: [u8; 32],
    },

    // 1
    /// Sets the new owner of a lending market.
    ///
    /// Accounts expected by this instruction:
    ///
    ///
    ///   0. `[writable]` Lending market account.
    ///   1. `[signer]` Current owner.
    SetLendingMarketOwner {
        /// The new owner
        new_owner: Pubkey,
    },

    // 2
    /// Initializes a new lending market reserve.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Reserve account - uninitialized.
    ///
    ///   1. `[]` Reserve liquidity SPL Token mint.
    ///   2. `[]` Reserve liquidity supply SPL Token account.
    ///   3. `[]` Reserve liquidity fee receiver.
    ///
    ///   4. `[]` Pyth product account  when is_lp is false
    ///           Any account when is_lp is true
    ///
    ///   5. `[]` Reserve liquidity pyth oracle account when is_lp is false
    ///           BridgePool account of bridge program when is_lp is true

    ///   6. `[]` Reserve liquidity larix oracle account when is_lp is false
    ///           LpPrice account of bridge program when is_lp is true

    ///   7. `[]` Reserve collateral SPL Token mint.
    ///
    ///   8. `[]` Reserve collateral token supply.
    ///   9  `[]` Lending market account.
    ///
    ///   10  `[signer]` Lending market owner.
    ///   11. `[]` Un_coll_supply_account
    ///
    ///   12  `[]` Clock sysvar.
    ///
    ///   13 `[]` Rent sysvar.
    ///   14 `[]` Token program id.

    InitReserve {
        /// Reserve configuration values
        total_mining_speed: u64,
        kink_util_rate: u64,
        use_pyth_oracle:bool,
        is_lp:bool,
    },

    // 3
    /// Accrue interest and update market price of liquidity on a reserve.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Reserve account.
    ///
    ///   1. `[]` Reserve liquidity oracle account.
    ///             Must be the Pyth price account specified at InitReserve.
    ///   2. `[]`  Larix oracle
    RefreshReserve,

    // 4
    /// Deposit liquidity into a reserve in exchange for collateral. Collateral represents a share
    /// of the reserve liquidity pool.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account.
    ///                     $authority can transfer $liquidity_amount.
    ///   1. `[writable]` Destination collateral token account.
    ///   2. `[writable]` Reserve account.
    ///   3. `[writable]` Reserve collateral SPL Token mint.
    ///   4. `[writable]` Reserve liquidity supply SPL Token account.
    ///   5. `[]` Lending market account.
    ///   6. `[]` Derived lending market authority.
    ///   7. `[signer]` User transfer authority ($authority).
    ///   8. `[]` Token program id.
    DepositReserveLiquidity {
        /// Amount of liquidity to deposit in exchange for collateral tokens
        liquidity_amount: u64,
    },

    // 5
    /// Redeem collateral from a reserve in exchange for liquidity.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source collateral token account.
    ///                     $authority can transfer $collateral_amount.
    ///   1. `[writable]` Reserve account.
    ///   2. `[writable]` Reserve collateral SPL Token mint.
    ///   3. `[writable]` Reserve liquidity supply SPL Token account.
    ///   4. `[]` Lending market account.
    ///   5. `[]` Derived lending market authority.
    ///   6. `[signer]` User transfer authority ($authority).
    ///   7. `[]` Token program id.
    ///
    ///   8. `[writable]` Destination liquidity token account.
    ///      or
    ///   8. `[writable]` Bridge pool info
    ///   9. `[]` Bridge program id
    ///   10.`[writable]` Bridge withdraw lp account
    RedeemReserveCollateral {
        /// Amount of collateral tokens to redeem in exchange for liquidity
        collateral_amount: u64,
    },

    // 6
    /// Initializes a new lending market obligation.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Obligation account - uninitialized.
    ///   1. `[]` Lending market account.
    ///   2. `[signer]` Obligation owner.
    ///   3. `[]` Token program id.
    InitObligation,

    // 7
    /// Refresh an obligation's accrued interest and collateral and liquidity prices. Requires
    /// refreshed reserves, as all obligation collateral deposit reserves in order, followed by all
    /// liquidity borrow reserves in order.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Obligation account.
    ///   .. `[]` Collateral deposit reserve accounts - refreshed, all, in order.
    ///   .. `[]` Liquidity borrow reserve accounts - refreshed, all, in order.
    RefreshObligation,

    // 8
    /// Deposit collateral to an obligation. Requires a refreshed reserve.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source collateral token account.
    ///                     Minted by deposit reserve collateral mint.
    ///                     $authority can transfer $collateral_amount.
    ///   1. `[writable]` Destination deposit reserve collateral supply SPL Token account.
    ///   2. `[]` Deposit reserve account - refreshed.
    ///   3. `[writable]` Obligation account.
    ///   4. `[]` Lending market account.
    ///   5. `[]` Derived lending market authority.
    ///   6. `[signer]` Obligation owner.
    ///   7. `[signer]` User transfer authority ($authority).
    ///   8. `[]` Token program id.
    DepositObligationCollateral {
        /// Amount of collateral tokens to deposit
        collateral_amount: u64,
    },

    // 9
    /// Withdraw collateral from an obligation. Requires a refreshed obligation and reserve.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source collateral token account.
    ///                     Minted by deposit reserve collateral mint.
    ///                     $authority can transfer $collateral_amount.
    ///   1. `[writable]` Destination deposit reserve collateral supply SPL Token account.
    ///   2. `[]` Withdraw reserve account - refreshed.
    ///   3. `[writable]` Obligation account.
    ///   4. `[]` Lending market account.
    ///   5. `[]` Derived lending market authority.
    ///   6. `[signer]` Obligation owner.
    ///   7. `[]` Token program id.
    WithdrawObligationCollateral {
        /// Amount of collateral tokens to withdraw - u64::MAX for up to 100% of deposited amount
        collateral_amount: u64,
    },

    // @TODO: rename cf. https://git.io/JOOE6
    // 10
    /// Borrow liquidity from a reserve by depositing collateral tokens. Requires a refreshed
    /// obligation and reserve.
    /// ::Useless
    ///     The current account will not be used.
    ///    It is used to make up the account number,
    ///    in order to keep the size of the current instruction is equals to liquidate obligation instruction,
    ///    to avoid the situation that the current transaction is successful but the liquidate cannot be performed
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source borrow reserve liquidity supply SPL Token account.
    ///   1. `[writable]` Destination liquidity token account.
    ///                     Minted by borrow reserve liquidity mint.
    ///   2. `[writable]` Borrow reserve account - refreshed.
    ///   3. `[writable]` Obligation account - refreshed.
    ///   4. `[]` Lending market account.
    ///   5. `[]` Derived lending market authority.
    ///   6. `[signer]` Obligation owner.
    ///   7. `[]` Token program id.
    ///   8. `[]` Borrow fee receiver
    ///   9. `[]` Larix oracle program account- Useless
    ///   10. `[]` Mine mint account - Useless
    BorrowObligationLiquidity {
        /// Amount of liquidity to borrow - u64::MAX for 100% of borrowing power
        liquidity_amount: u64,
        // @TODO: slippage constraint - https://git.io/JmV67
    },

    // 11
    /// Repay borrowed liquidity to a reserve. Requires a refreshed obligation and reserve.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account.
    ///                     Minted by repay reserve liquidity mint.
    ///                     $authority can transfer $liquidity_amount.
    ///   1. `[writable]` Destination repay reserve liquidity supply SPL Token account.
    ///   2. `[writable]` Repay reserve account - refreshed.
    ///   3. `[writable]` Obligation account - refreshed.
    ///   4. `[]` Lending market account.
    ///   5. `[signer]` User transfer authority ($authority).
    ///   6. `[]` Token program id.
    RepayObligationLiquidity {
        /// Amount of liquidity to repay - u64::MAX for 100% of borrowed amount
        liquidity_amount: u64,
    },

    // 12
    /// Repay borrowed liquidity to a reserve to receive collateral at a discount from an unhealthy
    /// obligation. Requires a refreshed obligation and reserves.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account.
    ///                     Minted by repay reserve liquidity mint.
    ///                     $authority can transfer $liquidity_amount.
    ///   1. `[writable]` Destination collateral token account.
    ///                     Minted by withdraw reserve collateral mint.
    ///   2. `[writable]` Repay reserve account - refreshed.
    ///   3. `[writable]` Repay reserve liquidity supply SPL Token account.
    ///   4. `[]` Withdraw reserve account - refreshed.
    ///   5. `[writable]` Withdraw reserve collateral supply SPL Token account.
    ///   6. `[writable]` Obligation account - refreshed.
    ///   7. `[]` Lending market account.
    ///   8. `[]` Derived lending market authority.
    ///   9. `[signer]` User transfer authority ($authority).
    ///   10 `[]` Clock sysvar.
    ///   11 `[]` Token program id.
    LiquidateObligation {
        /// Amount of liquidity to repay - u64::MAX for up to 100% of borrowed amount
        liquidity_amount: u64,
    },

    // 13
    /// Make a flash loan.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account.
    ///                     Minted by reserve liquidity mint.
    ///                     Must match the reserve liquidity supply.
    ///   1. `[writable]` Destination liquidity token account.
    ///                     Minted by reserve liquidity mint.
    ///   2. `[writable]` Reserve account.
    ///   3. `[writable]` Flash loan fee receiver account.
    ///                     Must match the reserve liquidity fee receiver.
    ///   4. `[writable]` Host fee receiver.
    ///   5. `[]` Lending market account.
    ///   6. `[]` Derived lending market authority.
    ///   7. `[]` Token program id.
    ///   8. `[]` Flash loan receiver program id.
    ///             Must implement an instruction that has tag of 0 and a signature of `(amount: u64)`
    ///             This instruction must return the amount to the source liquidity account.
    ///   9. `[signer]` Flash loan authority
    ///   .. `[any]` Additional accounts expected by the receiving program's `ReceiveFlashLoan` instruction.
    ///
    ///   The flash loan receiver program that is to be invoked should contain an instruction with
    ///   tag `0` and accept the total amount (including fee) that needs to be returned back after
    ///   its execution has completed.
    ///
    ///   Flash loan receiver should have an instruction with the following signature:
    ///
    ///   0. `[writable]` Source liquidity (matching the destination from above).
    ///   1. `[writable]` Destination liquidity (matching the source from above).
    ///   2. `[]` Token program id
    ///   .. `[any]` Additional accounts provided to the lending program's `FlashLoan` instruction above.
    ///   ReceiveFlashLoan {
    ///       // Amount that must be repaid by the receiver program
    ///       amount: u64
    ///   }
    FlashLoan {
        /// The amount that is to be borrowed - u64::MAX for up to 100% of available liquidity
        amount: u64,
        call_back_data: Vec<u8>
    },
    // 14
    ///
    ///
    SetConfig ,
    // 16
    /// 0. `[]` Mining account
    /// 1. `[signer]` Mining owner
    /// 2. `[]` Lending market account
    ///
    InitMining,

    // 17
    ///
    /// 0.  `[]` Mining account
    /// 1.. `[]` Reserves in mining

    RefreshMining,
    // 18
    /// 0. `[Writable]` Source account
    /// 1. `[Writable]` UnColl deposit supply SPL Token account.
    /// 2. `[Writable]` Mining account
    /// 3. `[]` Bonus account
    /// 4. `[]` Lending market account.
    /// 5. `[]` Mining owner.
    /// 6. `[signer]`   User transfer authority ($authority).
    /// 7. `[]` Token program id.
    DepositMining{
        amount:u64
    },
    // 19
    /// 0. `[writable]` Source account
    /// 1. `[writable]` UnColl deposit supply SPL Token account.
    /// 2. `[writable]` Mining account
    /// 3. `[writable]` Reserve account
    /// 4. `[]` Lending market account.
    /// 5. `[]` Derived lending market authority.
    /// 6. `[]` Mining owner.
    /// 7. `[]` Token program id.
    WithdrawMining{
        amount:u64
    },

    // 20
     /// 0. `[writable]` Mining account
     /// 1. `[]` Mine supply
     /// 2. `[]` Destination account
     /// 3. `[Signer]` Mining owner
     /// 4. `[]` Lending market info
     /// 5. `[]` Lending market authority
     /// 6. `[]` Token program id
     /// 7. `[]`
     ///     ... Reserves
    ClaimMiningMine,


    // 21
    /// 0. `[writable]` Obligation account
    /// 1. `[]` Mine supply
    /// 2. `[]` Destination account
    /// 3. `[Signer]` Obligation owner
    /// 4. `[]` Lending market info
    /// 5. `[]` Lending market authority
    /// 6. `[]` Token program id
    ClaimObligationMine,

    // 22
    /// 0. `[]` Source account (liquidity supply account)
    /// 1. `[]` Destination account receive owner fee
    /// 2. `[]` Lending market account
    /// 3. `[singer]` Lending market owner
    ClaimOwnerFee,

    // 23
    /// 0. `[Write]` Lending Market
    /// 1. `[Signer]` Pending owner
    ///
    ReceivePendingOwner,
    // 24
    ///
    ///   0. `[writable]` Reserve account.
    ///   1. `[]` Oracle account larix oracle or pyth price account .
    ///
    ///   ...
    ///
    ///   `[writable]` Reserve account.
    ///   `[]` Oracle account larix oracle or pyth price account
    ///
    RefreshReserves,

    // 25
    /// Repay borrowed liquidity to a reserve to receive collateral at a discount from an unhealthy
    /// obligation. Requires a refreshed obligation and reserves.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account.
    ///                     Minted by repay reserve liquidity mint.
    ///                     $authority can transfer $liquidity_amount.
    ///   1. `[writable]` Destination collateral token account.
    ///                     Minted by withdraw reserve collateral mint.
    ///   2. `[writable]` Repay reserve account - refreshed.
    ///   3. `[writable]` Repay reserve liquidity supply SPL Token account.
    ///   4. `[]` Withdraw reserve account - refreshed.
    ///   5. `[writable]` Withdraw reserve collateral supply SPL Token account.
    ///   6. `[writable]` Obligation account - refreshed.
    ///   7. `[]` Lending market account.
    ///   8. `[]` Derived lending market authority.
    ///   9. `[signer]` User transfer authority ($authority).
    ///   10 `[]` Token program id.
    LiquidateObligation2 {
        /// Amount of liquidity to repay - u64::MAX for up to 100% of borrowed amount
        liquidity_amount: u64,
    },

    // 26
    ///
    ///  0.  `[]` Token program
    ///  1.  `[]` Lending market account
    ///  2.  `[]` Lending market authority
    ///  3.  `[Signer]` Owner
    ///  4.  `[]` Mining account
    ///  5.  `[]` Obligation account.
    ///           After accounts pop if this account can not provided
    ///  6.  `[]` Destination account
    ///  7.  `[]` Source account
    ///
    ///  or
    ///
    ///  6.  `[]` Larix lock program
    ///  7.  `[]` Larix lock pool
    ///  8.  `[]` User larix info account
    ///
    ClaimMine{
        // claim times of user expected got: 100 equals 100%
        claim_times:u16,
        // the ratio of claim user's all mine token 10000 equals 100%
        claim_ratio:u16
    }
}

impl LendingInstruction {
    /// Unpacks a byte buffer into a [LendingInstruction](enum.LendingInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(LendingError::InstructionUnpackError)?;
        Ok(match tag {
            0 => {
                let (owner, rest) = unpack_pubkey(rest)?;
                let (quote_currency, _rest) = unpack_bytes32(rest)?;

                Self::InitLendingMarket {
                    owner,
                    quote_currency: *quote_currency,
                }

            }
            1 => {
                let (new_owner, _rest) = unpack_pubkey(rest)?;
                Self::SetLendingMarketOwner { new_owner }
            }
            2 => {
                let (_optimal_utilization_rate, rest) = unpack_u8(rest)?;
                let (_loan_to_value_ratio, rest) = unpack_u8(rest)?;
                let (_liquidation_bonus, rest) = unpack_u8(rest)?;
                let (_liquidation_threshold, rest) = unpack_u8(rest)?;
                let (_min_borrow_rate, rest) = unpack_u8(rest)?;
                let (_optimal_borrow_rate, rest) = unpack_u8(rest)?;
                let (_max_borrow_ratse, rest) = unpack_u8(rest)?;
                let (_borrow_fee_wad, rest) = unpack_u64(rest)?;
                let (_reserve_owner_fee_wad, rest) = unpack_u64(rest)?;
                let (_flash_loan_fee_wad, rest) = unpack_u64(rest)?;
                let (_host_fee_percentage, rest) = unpack_u8(rest)?;
                let (total_mining_speed,rest) = unpack_u64(rest)?;
                let (kink_util_rate,rest) = unpack_u64(rest)?;
                let (use_pyth_oracle,rest) = unpack_bool(rest)?;
                let (is_lp,_rest) = unpack_bool(rest)?;
                Self::InitReserve {
                    total_mining_speed,
                    kink_util_rate,
                    use_pyth_oracle,
                    is_lp
                }
            }
            3 => Self::RefreshReserve,
            4 => {
                let (liquidity_amount, _rest) = unpack_u64(rest)?;
                Self::DepositReserveLiquidity { liquidity_amount }
            }
            5 => {
                let (collateral_amount, _rest) = unpack_u64(rest)?;
                Self::RedeemReserveCollateral { collateral_amount }
            }
            6 => Self::InitObligation,
            7 => Self::RefreshObligation,
            8 => {
                let (collateral_amount, _rest) = unpack_u64(rest)?;
                Self::DepositObligationCollateral { collateral_amount }
            }
            9 => {
                let (collateral_amount, _rest) = unpack_u64(rest)?;
                Self::WithdrawObligationCollateral { collateral_amount }
            }
            10 => {
                let (liquidity_amount, _rest) = unpack_u64(rest)?;
                Self::BorrowObligationLiquidity { liquidity_amount }
            }
            11 => {
                let (liquidity_amount, _rest) = unpack_u64(rest)?;
                Self::RepayObligationLiquidity { liquidity_amount }
            }
            12 => {
                let (liquidity_amount, _rest) = unpack_u64(rest)?;
                Self::LiquidateObligation { liquidity_amount }
            }
            13 => {
                let (amount, rest) = unpack_u64(rest)?;
                let mut call_back_data =  Vec::with_capacity(rest.len());
                call_back_data.extend_from_slice(rest);
                Self::FlashLoan { amount ,call_back_data}
            }
            14 => {
                Self::SetConfig
            }
            16 => {
                Self::InitMining
            }
            17 => {
                Self::RefreshMining
            }
            18 => {
                let (amount, _rest) = unpack_u64(rest)?;
                Self::DepositMining { amount }
            }
            19 => {
                let (amount, _rest) = unpack_u64(rest)?;
                Self::WithdrawMining { amount }
            }
            20 => {
                Self::ClaimMiningMine
            }
            21 => {
                Self::ClaimObligationMine
            }
            22 => {
                Self::ClaimOwnerFee
            }
            23 => {
                Self::ReceivePendingOwner
            }
            24 => {
                Self::RefreshReserves
            }
            25 => {
                let (liquidity_amount, _rest) = unpack_u64(rest)?;
                Self::LiquidateObligation2 {liquidity_amount}
            }
            26 => {
                let (subsidy_times, rest) = unpack_u16(rest)?;
                let (claim_ratio, _rest) = unpack_u16(rest)?;
                Self::ClaimMine {
                    claim_times: subsidy_times,
                    claim_ratio
                }
            }
            _ => {
                msg!("Instruction cannot be unpacked");
                return Err(LendingError::InstructionUnpackError.into());
            }
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match *self {
            Self::RefreshReserves => {
                buf.push(24)
            }
            Self::DepositReserveLiquidity { liquidity_amount } => {
                buf.push(4);
                buf.extend_from_slice(&liquidity_amount.to_le_bytes());
            }
            Self::RedeemReserveCollateral { collateral_amount } => {
                buf.push(5);
                buf.extend_from_slice(&collateral_amount.to_le_bytes());
            }
            Self::InitObligation => {
                buf.push(6);
            }
            Self::RefreshObligation => {
                buf.push(7);
            }
            Self::DepositObligationCollateral { collateral_amount } => {
                buf.push(8);
                buf.extend_from_slice(&collateral_amount.to_le_bytes());
            }
            Self::WithdrawObligationCollateral { collateral_amount } => {
                buf.push(9);
                buf.extend_from_slice(&collateral_amount.to_le_bytes());
            }
            Self::BorrowObligationLiquidity { liquidity_amount } => {
                buf.push(10);
                buf.extend_from_slice(&liquidity_amount.to_le_bytes());
            }
            Self::RepayObligationLiquidity { liquidity_amount } => {
                buf.push(11);
                buf.extend_from_slice(&liquidity_amount.to_le_bytes());
            }
            Self::InitMining => {
                buf.push(16);
            }
            Self::RefreshMining => {
                buf.push(17);
            }
            Self::DepositMining { amount } => {
                buf.push(18);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::WithdrawMining { amount } => {
                buf.push(19);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::ClaimMiningMine => {
                buf.push(20);
            }
            Self::ClaimObligationMine => {
                buf.push(21);
            }
            Self::LiquidateObligation2{liquidity_amount} =>{
                buf.push(25);
                buf.extend_from_slice(&liquidity_amount.to_le_bytes());
            }
            Self::ClaimMine { claim_times, claim_ratio} => {
                buf.push(26);
                buf.extend_from_slice(&claim_times.to_le_bytes());
                buf.extend_from_slice(&claim_ratio.to_le_bytes());
            }
            _ => {
                // TODO: implementation
            }
        }
        buf
    }
}
pub fn refresh_reserves(
    program_id:Pubkey,
    reserves:Vec<Pubkey>,
    oracles:Vec<Pubkey>
) -> Instruction {
    let mut accounts = Vec::with_capacity(reserves.len()*2);
    for i in 0..reserves.len(){
        accounts.push(AccountMeta::new(reserves[i],false));
        accounts.push(AccountMeta::new_readonly(oracles[i],false));
    }
    Instruction {
        program_id,
        accounts,
        data: LendingInstruction::RefreshReserves.pack(),
    }
}
#[allow(clippy::too_many_arguments)]
pub fn deposit_reserve_liquidity(
    program_id: Pubkey,
    liquidity_amount: u64,
    source_liquidity_pubkey: Pubkey,
    destination_collateral_pubkey: Pubkey,
    reserve_pubkey: Pubkey,
    reserve_collateral_mint_pubkey: Pubkey,
    reserve_liquidity_supply_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    lending_market_authority_pubkey:Pubkey,
    user_transfer_authority_pubkey: Pubkey
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source_liquidity_pubkey, false),
            AccountMeta::new(destination_collateral_pubkey, false),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new(reserve_collateral_mint_pubkey, false),
            AccountMeta::new(reserve_liquidity_supply_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(lending_market_authority_pubkey, false),
            AccountMeta::new_readonly(user_transfer_authority_pubkey, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LendingInstruction::DepositReserveLiquidity { liquidity_amount }.pack(),
    }
}
/// Creates a 'RedeemReserveCollateral' instruction.
#[allow(clippy::too_many_arguments)]
pub fn redeem_reserve_collateral(
    program_id: Pubkey,
    collateral_amount: u64,
    source_collateral_pubkey: Pubkey,
    destination_liquidity_pubkey: Pubkey,
    reserve_pubkey: Pubkey,
    reserve_collateral_mint_pubkey: Pubkey,
    reserve_liquidity_supply_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    lending_market_authority_pubkey:Pubkey,
    user_transfer_authority_pubkey: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source_collateral_pubkey, false),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new(reserve_collateral_mint_pubkey, false),
            AccountMeta::new(reserve_liquidity_supply_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(lending_market_authority_pubkey, false),
            AccountMeta::new_readonly(user_transfer_authority_pubkey, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(destination_liquidity_pubkey, false),
        ],
        data: LendingInstruction::RedeemReserveCollateral { collateral_amount }.pack(),
    }
}
/// Creates an 'InitObligation' instruction.
#[allow(clippy::too_many_arguments)]
pub fn init_obligation(
    program_id: Pubkey,
    obligation_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    obligation_owner_pubkey: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(obligation_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(obligation_owner_pubkey, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LendingInstruction::InitObligation.pack(),
    }
}

/// Creates a 'RefreshObligation' instruction.
#[allow(clippy::too_many_arguments)]
pub fn refresh_obligation(
    program_id: Pubkey,
    obligation_pubkey: Pubkey,
    reserve_pubkeys: Vec<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(obligation_pubkey, false)
    ];
    accounts.extend(
        reserve_pubkeys
            .into_iter()
            .map(|pubkey| AccountMeta::new_readonly(pubkey, false)),
    );
    Instruction {
        program_id,
        accounts,
        data: LendingInstruction::RefreshObligation.pack(),
    }
}

/// Creates a 'DepositObligationCollateral' instruction.
#[allow(clippy::too_many_arguments)]
pub fn deposit_obligation_collateral(
    program_id: Pubkey,
    collateral_amount: u64,
    source_collateral_pubkey: Pubkey,
    destination_collateral_pubkey: Pubkey,
    deposit_reserve_pubkey: Pubkey,
    obligation_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    lending_market_authority_pubkey:Pubkey,
    obligation_owner_pubkey: Pubkey,
    user_transfer_authority_pubkey: Pubkey,
    reserve_pubkeys: Vec<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(source_collateral_pubkey, false),
        AccountMeta::new(destination_collateral_pubkey, false),
        AccountMeta::new_readonly(deposit_reserve_pubkey, false),
        AccountMeta::new(obligation_pubkey, false),
        AccountMeta::new_readonly(lending_market_pubkey, false),
        AccountMeta::new_readonly(lending_market_authority_pubkey, false),
        AccountMeta::new_readonly(obligation_owner_pubkey, true),
        AccountMeta::new_readonly(user_transfer_authority_pubkey, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    accounts.extend(
        reserve_pubkeys
            .into_iter()
            .map(|pubkey| AccountMeta::new_readonly(pubkey, false)),
    );
    Instruction {
        program_id,
        accounts,
        data: LendingInstruction::DepositObligationCollateral { collateral_amount }.pack(),
    }
}
/// Creates a 'WithdrawObligationCollateral' instruction.
#[allow(clippy::too_many_arguments)]
pub fn withdraw_obligation_collateral(
    program_id: Pubkey,
    collateral_amount: u64,
    source_collateral_pubkey: Pubkey,
    destination_collateral_pubkey: Pubkey,
    withdraw_reserve_pubkey: Pubkey,
    obligation_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    obligation_owner_pubkey: Pubkey,
) -> Instruction {
    let (lending_market_authority_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[&lending_market_pubkey.to_bytes()[..PUBKEY_BYTES]],
        &program_id,
    );
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source_collateral_pubkey, false),
            AccountMeta::new(destination_collateral_pubkey, false),
            AccountMeta::new_readonly(withdraw_reserve_pubkey, false),
            AccountMeta::new(obligation_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(lending_market_authority_pubkey, false),
            AccountMeta::new_readonly(obligation_owner_pubkey, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LendingInstruction::WithdrawObligationCollateral { collateral_amount }.pack(),
    }
}
/// Creates a 'BorrowObligationLiquidity' instruction.
#[allow(clippy::too_many_arguments)]
pub fn borrow_obligation_liquidity(
    program_id: Pubkey,
    liquidity_amount: u64,
    source_liquidity_pubkey: Pubkey,
    destination_liquidity_pubkey: Pubkey,
    borrow_reserve_pubkey: Pubkey,
    borrow_reserve_liquidity_fee_receiver_pubkey: Pubkey,
    obligation_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    lending_market_authority_pubkey:Pubkey,
    obligation_owner_pubkey: Pubkey,
    larix_oracle_program_pubkey:Pubkey,
    mine_mint_pubkey:Pubkey
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(source_liquidity_pubkey, false),
        AccountMeta::new(destination_liquidity_pubkey, false),
        AccountMeta::new(borrow_reserve_pubkey, false),

        AccountMeta::new(obligation_pubkey, false),
        AccountMeta::new_readonly(lending_market_pubkey, false),
        AccountMeta::new_readonly(lending_market_authority_pubkey, false),

        AccountMeta::new_readonly(obligation_owner_pubkey, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(borrow_reserve_liquidity_fee_receiver_pubkey, false),
        AccountMeta::new_readonly(larix_oracle_program_pubkey, false),
        AccountMeta::new_readonly(mine_mint_pubkey, false),
    ];
    Instruction {
        program_id,
        accounts,
        data: LendingInstruction::BorrowObligationLiquidity { liquidity_amount }.pack(),
    }
}
/// Creates a `RepayObligationLiquidity` instruction
#[allow(clippy::too_many_arguments)]
pub fn repay_obligation_liquidity(
    program_id: Pubkey,
    liquidity_amount: u64,
    source_liquidity_pubkey: Pubkey,
    destination_liquidity_pubkey: Pubkey,
    repay_reserve_pubkey: Pubkey,
    obligation_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    user_transfer_authority_pubkey: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source_liquidity_pubkey, false),
            AccountMeta::new(destination_liquidity_pubkey, false),
            AccountMeta::new(repay_reserve_pubkey, false),
            AccountMeta::new(obligation_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(user_transfer_authority_pubkey, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: LendingInstruction::RepayObligationLiquidity { liquidity_amount }.pack(),
    }
}
pub fn claim_obligation_mine(
    program_id: Pubkey,
    obligation_pubkey:Pubkey,
    mine_supply_pubkey:Pubkey,
    destination_account_pubkey:Pubkey,
    obligation_owner:Pubkey,
    lending_market_pubkey:Pubkey,
    lending_market_authority_pubkey:Pubkey,
) ->Instruction {
    let accounts = vec![
        AccountMeta::new(obligation_pubkey, false),
        AccountMeta::new(mine_supply_pubkey, false),
        AccountMeta::new(destination_account_pubkey, false),
        AccountMeta::new_readonly(obligation_owner, false),
        AccountMeta::new_readonly(lending_market_pubkey, false),
        AccountMeta::new_readonly(lending_market_authority_pubkey, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id,
        accounts,
        data: LendingInstruction::ClaimObligationMine.pack(),
    }
}

