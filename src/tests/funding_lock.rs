use crate::ContractUtil;






#[test]
fn test_funding_lock(){
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");

}