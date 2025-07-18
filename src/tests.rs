use crate::tools::Tools;
use mcplease::traits::AsToolsList;

#[test]
fn tools_doesnt_panic() {
    Tools::tools_list();
}
