# Debugging notes

Since there is no PR to comment on, adding some debugging notes 
for the `update/add-cw-orch` branch here.

## To reproduce

```bash
cd examples/contracts/cw20-base
cargo build
cargo test
```

(Note I can get build to pass by commented out some parts, but then test fails...)

I was trying to debug the root cause of:

```
error[E0119]: conflicting implementations of trait `From<ContractExecMsg>` for type `ContractExecMsg`
  --> contracts/cw20-base/src/contract.rs:76:1
   |
76 | #[contract]
   | ^^^^^^^^^^^
   |
   = note: conflicting implementation in crate `core`:
           - impl<T> From<T> for T;
   = note: this error originates in the attribute macro `contract` (in Nightly builds, run with -Z macro-backtrace for more info)
```

I added some print statements and got this output:

```
Exec
whole_type_name: < cw20_allowances :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec
contract_enum_name: ContractExecMsg
variant: Allowances
From<< cw20_allowances :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec> for ContractExecMsg

whole_type_name: < cw20_marketing :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec
contract_enum_name: ContractExecMsg
variant: Marketing
From<< cw20_marketing :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec> for ContractExecMsg

whole_type_name: < cw20_minting :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec
contract_enum_name: ContractExecMsg
variant: Minting
From<< cw20_minting :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec> for ContractExecMsg
```

## Proposed Solutions

Maybe there is some better way to name this object than `< cw20_minting :: sv :: Api as sylvia :: types :: InterfaceApi > :: Exec>`? It must have a concrete type somewhere, right? 

Let's check out what we have by running `cargo doc --open`.
(After commenting out the `impl_into_underlying` stuff that broke the compilation)
Dig down into `contract` ... `sv` ...

For example `doc/cw20_base/contract/sv/enum.ContractExecMsg.html`

I noticed 2 things:

1. It has 4 variants, but only 3 are called previously, not the local messages
2. This is for implementations of remote interfaces.

This will require upstream changes to work with most likely, so maybe we can reduce scope:

1. Comment out the `impl_into_underlying` stuff
2. Call the orchestrator with the heavier `.query(QueryMsg::Allowances { ... })` syntax for now
3. Ensure this works in mt and local deploy
4. See if we get the nicer function syntax for the locally implemented exec messages, eg, `contract.transfer(...)`

The goal wouldn't then be "full powered `cw-orch`" for Sylvia contracts, but at least
"some use of `cw-orch` for Sylvia contracts". This would let us write and test deploy scripts
in `cw-orch` even if most of the tests are using sylvia's mt framework.

It's not perfect, but it would open the door to integration.