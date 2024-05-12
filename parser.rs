pub struct Parser {
    pub tokens: Vec<String>,
    pub cursor: usize
}

#[derive(PartialEq,Debug,Clone)]
pub enum DataType {
    INT,
    STRING
}

#[derive(PartialEq,Debug,Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub size: u32,
    pub nullable: bool
}

#[derive(PartialEq,Debug)]
pub enum ASTNode {
    CreateStatement { table_name: String, columns_to_add: Vec<Column>, next: Option<Box<ASTNode>>},
    DropStatement { table_name: String, next: Option<Box<ASTNode>> },
    SelectStatement { table_name: String, distinct: bool, columns: Vec<String>, next: Option<Box<ASTNode>>},
    InsertStatement { table_name: String, columns: Vec<String>, values: Vec<String>, next: Option<Box<ASTNode>>},
    ShowTablesStatement { next: Option<Box<ASTNode>> },
    AlterTableStatement { table_name: String, columns_to_add: Vec<Column>, next: Option<Box<ASTNode>> },
    TruncateTableStatement { table_name: String, next: Option<Box<ASTNode>> }
}

impl Parser {
    fn expect(&mut self, expected: &String)
    {
        assert_eq!(self.current(), expected);
    }

    fn accept(&mut self, expected: &String) -> bool
    {
        return self.current() == expected;
    }
    
    fn accept_indentation(&mut self)
    {
        if self.current() == &" ".to_string() || self.current() == &"\t".to_string() {
            self.next();
        }
    }

    fn current(&mut self) -> &String {
        return self.tokens.get(self.cursor).unwrap();
    }

    fn next(&mut self) {
        if self.cursor < self.tokens.len()-1 {
            self.cursor += 1;
        }
    }

    fn tokenizer(&mut self, input: String) -> Vec<String> {
        let mut list = Vec::new();
        let mut word = String::new();
        for character in input.chars() {
            if character == '\n' {
                if word.len() > 0 {
                    list.push(word.clone());
                    word.clear();
                }
                continue;
            }
            else if character == ';' || character == ' ' || character == '\t' || character == '(' || character == ')' || character == ',' || character == '*' {
                if word.len() > 0 {
                    list.push(word.clone());
                    word.clear();
                }
                list.push(character.to_string());
            }
            else {
                word.push(character);
            }
        }
        return list;
    }

    fn parseNewColumn(&mut self) -> Option<Column> {
        let column_name = self.current().clone();
        self.next();
        self.accept_indentation();
        //println!("Parser: Column named '{}'", column_name);
        if self.accept(&"INT".to_string()) {
            self.next();
            self.accept_indentation();
            return Some(Column {
                name: column_name,
                data_type: DataType::INT,
                size: 0, // TODO: Set this to 4 bytes
                nullable: false // TODO: Set this to correct value
            });
        } else if self.accept(&"VARCHAR".to_string()) {
            self.next();
            self.accept_indentation();
            self.expect(&"(".to_string());
            self.next();
            let data_size_string: String = self.current().clone(); 
            let data_size: u32 = data_size_string.parse().unwrap();
            self.next();
            self.accept_indentation();
            self.expect(&")".to_string());
            self.next();
            self.accept_indentation();
            return Some(Column {
                name: column_name,
                data_type: DataType::STRING,
                size: data_size, // TODO: Set this to 4 bytes
                nullable: false // TODO: Set this to correct value
            });
        } else {
            panic!("Unexpected data type");
        }
    }

    fn parse_create_table(&mut self) -> Option<Box<ASTNode>> {
        let mut columns_to_add = Vec::new();
        self.expect(&"CREATE".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"TABLE".to_string());
        self.next();
        self.accept_indentation();
        let table_name = self.current().clone();
        self.next();
        self.accept_indentation();
        if self.accept(&"(".to_string()) {
            self.next();
            while self.current() != &")".to_string() {
                self.accept_indentation();
                let newColumn = self.parseNewColumn();
                if newColumn.is_some() {
                    columns_to_add.push(newColumn.unwrap());
                    if self.accept(&",".to_string()) {
                        self.next();
                        self.accept_indentation();
                    }
                }
            }
            self.expect(&")".to_string());
            self.next();
        }
        self.accept_indentation();
        self.expect(&";".to_string());
        self.next();
        println!("Parser: Create table named '{}'", table_name);
        let next = self.parse();
        return Some(Box::new(ASTNode::CreateStatement {
            table_name: table_name,
            columns_to_add: columns_to_add,
            next: next
        }));
    }

    fn parse_insert_statement(&mut self) -> Option<Box<ASTNode>> {
        self.expect(&"INSERT".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"INTO".to_string());
        self.next();
        self.accept_indentation();
        let table_name = self.current().clone();
        self.next();
        self.accept_indentation();
        self.expect(&"(".to_string());
        self.next();
        let mut columns = Vec::new();
        while self.current() != &")".to_string() {
            self.accept_indentation();
            columns.push(self.current().clone());
            self.next();
            if self.accept(&",".to_string()) {
                self.next();
            }
        }
        self.expect(&")".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"VALUES".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"(".to_string());
        self.next();
        let mut values = Vec::new();
        while self.current() != &")".to_string() {
            values.push(self.current().clone());
            self.next();
            if self.accept(&",".to_string()) {
                self.next();
            }
            self.accept_indentation();
        }
        self.expect(&")".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&";".to_string());
        self.next();
        println!("Parser: Insert into table named '{}'", table_name);
        let next = self.parse();
        return Some(Box::new(ASTNode::InsertStatement { table_name: table_name, columns: columns, values: values, next: next }));
    }

    fn parse_select_statement(&mut self) -> Option<Box<ASTNode>> {
        let mut distinct: bool = false;
        self.expect(&"SELECT".to_string());
        self.next();
        self.accept_indentation();
        if self.accept(&"DISTINCT".to_string()) {
            self.next();
            self.accept_indentation();
            distinct = true;
        }
        let mut columns = Vec::new();
        while self.current() != &"FROM".to_string() {
            columns.push(self.current().clone());
            self.next();
            self.accept_indentation();
            if self.accept(&",".to_string()) {
                self.next();
                self.accept_indentation();
            }
        }
        self.expect(&"FROM".to_string());
        self.next();
        self.accept_indentation();
        let table_name = self.current().clone();
        self.next();
        self.accept_indentation();
        self.expect(&";".to_string());
        self.next();
        println!("Parser: Select from table named '{}'", table_name);
        let next = self.parse();
        return Some(Box::new(ASTNode::SelectStatement {
            table_name: table_name,
            distinct: distinct,
            columns: columns,
            next: next
        }));
    }

    fn parse_drop_table(&mut self) -> Option<Box<ASTNode>> {
        self.expect(&"DROP".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"TABLE".to_string());
        self.next();
        self.accept_indentation();
        let table_name = self.current().clone();
        self.next();
        self.accept_indentation();
        self.expect(&";".to_string());
        self.next();
        println!("Parser: Drop table named '{}'", table_name);
        let next = self.parse();
        return Some(Box::new(ASTNode::DropStatement { table_name: table_name, next: next }));
    }

    fn parse_show_tables(&mut self) -> Option<Box<ASTNode>> {
        self.expect(&"SHOW".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"TABLES".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&";".to_string());
        self.next();
        println!("Parser: Show tables");
        let next = self.parse();
        return Some(Box::new(ASTNode::ShowTablesStatement { next: next }));
    }

    fn parse_alter_statement(&mut self) -> Option<Box<ASTNode>> {
        let mut columns = Vec::new();
        self.expect(&"ALTER".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"TABLE".to_string());
        self.next();
        self.accept_indentation();
        let table_name = self.current().clone();
        self.next();
        self.accept_indentation();
        self.expect(&"ADD".to_string());
        self.next();
        while self.current() != &";".to_string() {
            self.accept_indentation();
            let newColumn = self.parseNewColumn();
            if newColumn.is_some() {
                columns.push(newColumn.unwrap());
                if self.accept(&",".to_string()) {
                    self.next();
                    self.accept_indentation();
                }
            }
        }
        self.next();
        println!("Parser: Alter table named '{}'", table_name);
        let next = self.parse();
        return Some(Box::new(ASTNode::AlterTableStatement {
            table_name: table_name,
            columns_to_add: columns,
            next: next
        }));
    }

    fn parseTruncateTable(&mut self) -> Option<Box<ASTNode>> {
        self.expect(&"TRUNCATE".to_string());
        self.next();
        self.accept_indentation();
        self.expect(&"TABLE".to_string());
        self.next();
        self.accept_indentation();
        let table_name = self.current().clone();
        self.next();
        self.accept_indentation();
        self.expect(&";".to_string());
        self.next();
        println!("Parser: Truncate table named '{}'", table_name);
        let next = self.parse();
        return Some(Box::new(ASTNode::TruncateTableStatement { table_name: table_name, next: next }));
    }

    fn parse(&mut self) -> Option<Box<ASTNode>> {
        if self.accept(&"CREATE".to_string())
        {
            return self.parse_create_table();
        }
        else if self.accept(&"SHOW".to_string())
        {
            return self.parse_show_tables();
        }
        else if self.accept(&"SELECT".to_string())
        {
            return self.parse_select_statement();
        }
        else if self.accept(&"INSERT".to_string())
        {
            return self.parse_insert_statement();
        }
        else if self.accept(&"DROP".to_string())
        {
            return self.parse_drop_table();
        }
        else if self.accept(&"ALTER".to_string())
        {
            return self.parse_alter_statement();
        }
        else if self.accept(&"TRUNCATE".to_string())
        {
            return self.parseTruncateTable();
        }
        return None;
    }
    
    pub fn generate_ast(&mut self, query: String) -> Option<Box<ASTNode>> {
        //self.query = query;
        self.tokens = self.tokenizer(query);
        let top = self.parse();
        return top;
    }
}