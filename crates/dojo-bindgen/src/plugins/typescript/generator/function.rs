use cainome::parser::tokens::Function;
use convert_case::{Case, Casing};
use dojo_world::contracts::naming;

use crate::{error::BindgenResult, plugins::BindgenContractGenerator, DojoContract};

use super::JsType;

pub(crate) struct TsFunctionGenerator;
impl TsFunctionGenerator {
    fn check_imports(&self, buffer: &mut Vec<String>) {
        if !buffer.iter().any(|b| b.contains("import { DojoProvider } from ")) {
            buffer.insert(0, "import { DojoProvider } from \"@dojoengine/core\";".to_owned());
            buffer.insert(1, "import { Account } from \"starknet\";".to_owned());
            buffer.insert(2, "import * as models from \"./models.gen\";\n".to_owned());
        }
    }

    fn setup_function_wrapper_start(
        &self,
        contract: &DojoContract,
        buffer: &mut Vec<String>,
    ) -> usize {
        let fn_wrapper = format!("export async function setupWorld(provider: DojoProvider) {{\n\tconst contractName = \"{}\";\n", naming::get_name_from_tag(&contract.tag));

        if !buffer.iter().any(|b| b.contains(&fn_wrapper)) {
            buffer.push(fn_wrapper.clone());
        }

        buffer.iter().position(|b| b.contains(&fn_wrapper)).unwrap()
    }

    fn generate_system_function(&self, token: &Function) -> String {
        format!(
            "\tconst {} = async ({}) => {{
\t\ttry {{
\t\t\treturn await provider.execute(\n
\t\t\t\taccount,
\t\t\t\t{{
\t\t\t\t\tcontractName,
\t\t\t\t\tentryPoint: \"{}\",
\t\t\t\t\tcalldata: [{}],
\t\t\t\t}}
\t\t\t);
\t\t}} catch (error) {{
\t\t\tconsole.error(error);
\t\t}}
\t}};\n",
            token.name.to_case(Case::Camel),
            self.format_function_inputs(token),
            token.name,
            self.format_function_calldata(token)
        )
    }

    fn format_function_inputs(&self, token: &Function) -> String {
        let inputs = vec!["account: Account".to_owned()];
        token
            .inputs
            .iter()
            .fold(inputs, |mut acc, input| {
                acc.push(format!("{}: {}", input.0, JsType::from(input.1.type_name().as_str())));
                acc
            })
            .join(", ")
    }

    fn format_function_calldata(&self, token: &Function) -> String {
        token
            .inputs
            .iter()
            .fold(Vec::new(), |mut acc, input| {
                acc.push(format!("{}", input.0));
                acc
            })
            .join(", ")
    }

    fn append_function_body(&self, idx: usize, buffer: &mut Vec<String>, body: String) {
        // check if function was already appended to body, if so, append after other functions
        let pos = if buffer.len() - idx > 2 { buffer.len() - 2 } else { idx };

        buffer.insert(pos + 1, body);
    }

    fn setup_function_wrapper_end(&self, token: &Function, buffer: &mut Vec<String>) {
        let return_token = "\treturn {";
        if !buffer.iter().any(|b| b.contains(return_token)) {
            buffer
                .push(format!("\treturn {{\n\t\t{},\n\t}};\n}}", token.name.to_case(Case::Camel)));
            return;
        }

        let pos = buffer.iter().position(|b| b.contains(return_token)).unwrap();
        if let Some(st) = buffer.get_mut(pos) {
            let indices = st.match_indices(",").map(|(i, _)| i).collect::<Vec<usize>>();
            let append_after = indices[indices.len() - 1] + 1;
            st.insert_str(append_after, &format!("\n\t\t{},", token.name.to_case(Case::Camel)));
        }
    }
}

impl BindgenContractGenerator for TsFunctionGenerator {
    fn generate(
        &self,
        contract: &DojoContract,
        token: &Function,
        buffer: &mut Vec<String>,
    ) -> BindgenResult<String> {
        self.check_imports(buffer);
        let idx = self.setup_function_wrapper_start(contract, buffer);
        self.append_function_body(idx, buffer, self.generate_system_function(token));
        self.setup_function_wrapper_end(token, buffer);
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use cainome::parser::{
        tokens::{CoreBasic, Function, Token},
        TokenizedAbi,
    };

    use super::TsFunctionGenerator;
    use crate::{plugins::BindgenContractGenerator, DojoContract};

    #[test]
    fn test_check_imports() {
        let generator = TsFunctionGenerator {};
        let mut buff: Vec<String> = Vec::new();

        // check imports are added only once
        generator.check_imports(&mut buff);
        assert_eq!(buff.len(), 3);
        generator.check_imports(&mut buff);
        assert_eq!(buff.len(), 3);
    }

    #[test]
    fn test_setup_function_wrapper_start() {
        let generator = TsFunctionGenerator {};
        let mut buff: Vec<String> = Vec::new();
        let idx = generator.setup_function_wrapper_start(&create_dojo_contract(), &mut buff);

        assert_eq!(buff.len(), 1);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_generate_system_function() {
        let generator = TsFunctionGenerator {};
        let mut buff: Vec<String> = Vec::new();
        let function = create_change_theme_function();
        let expected = "\tconst changeTheme = async (account: Account, value: number) => {
\t\ttry {
\t\t\treturn await provider.execute(\n
\t\t\t\taccount,
\t\t\t\t{
\t\t\t\t\tcontractName,
\t\t\t\t\tentryPoint: \"change_theme\",
\t\t\t\t\tcalldata: [value],
\t\t\t\t}
\t\t\t);
\t\t} catch (error) {
\t\t\tconsole.error(error);
\t\t}
\t};\n";

        assert_eq!(expected, generator.generate_system_function(&function))
    }

    #[test]
    fn test_format_function_inputs() {
        let generator = TsFunctionGenerator {};
        let function = create_change_theme_function();
        let expected = "account: Account, value: number";
        assert_eq!(expected, generator.format_function_inputs(&function))
    }

    #[test]
    fn test_format_function_calldata() {
        let generator = TsFunctionGenerator {};
        let function = create_change_theme_function();
        let expected = "value";
        assert_eq!(expected, generator.format_function_calldata(&function))
    }

    #[test]
    fn test_append_function_body() {
        let generator = TsFunctionGenerator {};
        let mut buff = vec!["import".to_owned(), "function wrapper".to_owned()];

        generator.append_function_body(1, &mut buff, "function body".to_owned());

        assert_eq!(buff[2], "function body".to_owned());
    }

    #[test]
    fn test_setup_function_wrapper_end() {
        let generator = TsFunctionGenerator {};
        let mut buff: Vec<String> = Vec::new();

        generator.setup_function_wrapper_end(&create_change_theme_function(), &mut buff);

        let expected = "\treturn {
\t\tchangeTheme,
\t};
}";

        assert_eq!(1, buff.len());
        assert_eq!(expected, buff[0]);

        generator.setup_function_wrapper_end(&create_increate_global_counter_function(), &mut buff);
        let expected_2 = "\treturn {
\t\tchangeTheme,
\t\tincreaseGlobalCounter,
\t};
}";
        assert_eq!(1, buff.len());
        assert_eq!(expected_2, buff[0]);
    }

    #[test]
    fn test_it_generates_function() {
        let generator = TsFunctionGenerator {};
        let mut buffer = Vec::new();
        let change_theme = create_change_theme_function();

        let _ = generator.generate(&create_dojo_contract(), &change_theme, &mut buffer);
        assert_eq!(buffer.len(), 6);
        let increase_global_counter = create_increate_global_counter_function();
        let _ = generator.generate(&create_dojo_contract(), &increase_global_counter, &mut buffer);
        assert_eq!(buffer.len(), 7);
    }

    fn create_change_theme_function() -> Function {
        create_test_function(
            "change_theme",
            vec![(
                "value".to_owned(),
                Token::CoreBasic(CoreBasic { type_path: "core::integer::u8".to_owned() }),
            )],
        )
    }

    fn create_increate_global_counter_function() -> Function {
        create_test_function("increase_global_counter", vec![])
    }

    fn create_test_function(name: &str, inputs: Vec<(String, Token)>) -> Function {
        Function {
            name: name.to_owned(),
            state_mutability: cainome::parser::tokens::StateMutability::External,
            inputs,
            outputs: vec![],
            named_outputs: vec![],
        }
    }

    fn create_dojo_contract() -> DojoContract {
        DojoContract {
            tag: "onchain_dash-actions".to_owned(),
            tokens: TokenizedAbi::default(),
            systems: vec![],
        }
    }
}