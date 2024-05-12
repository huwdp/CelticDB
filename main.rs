use std::env;
use std::fs;
use std::collections::HashMap;
pub mod parser;

use parser::Parser;
use parser::ASTNode;
use parser::DataType;
use parser::Column;

#[derive(Clone)]
enum Cell {
    INT(i32, u32, bool),
    STRING(String, u32, bool)
}

struct Statement {
    query: String,
    correct: bool,
    errors: Vec<Error>
}

struct Error {
    type1: i32,
    message: String
}

struct Table {
    columns: Vec<Box<Column>>,
    rows: Vec<Vec<Cell>>,
    row_count: i32,
    cursor: i32
}

impl Table {
    pub fn insert_row(&mut self, row: Vec<Cell>) {
        self.rows.push(row);
    }

    pub fn delete_row(&mut self, index: i32) {
        self.rows.remove(index as usize);
    }

    pub fn find_column(&self, name: String) -> Option<&Box<Column>> {
        for column in &self.columns {
            if column.name == name {
                return Some(column);
            }
        }
        return None;
    }
}

struct Database {
    tables: HashMap<String, Box<Table>>
}

impl Database {
    pub fn compareEqCells(&self, cell1: &Cell, cell2: &Cell) -> bool {
        match cell1 {
            Cell::INT(value1, _, _) => {
                match cell2 {
                    Cell::INT(value2, _, _) => {
                        return value1 == value2;
                    },
                    Cell::STRING(_, _, _) => {
                        return false;
                    }
                }
            },
            Cell::STRING(value1, _, _) => {
                match cell2 {
                    Cell::INT(_, _, _) => {
                        return false;
                    },
                    Cell::STRING(value2, _, _) => {
                        return value1 == value2;
                    }
                }
            }
        }
    }

    pub fn create_table(&mut self, name: String) {
        self.tables.insert(name, Box::new(Table { cursor: 0, row_count: 0, columns: Vec::new(), rows: Vec::new()}));
    }

    pub fn drop_table(&mut self, name: String) {
        self.tables.remove(&name);
    }

    pub fn truncate_table(&mut self, name: String) {
        let table = self.tables.get_mut(&name).expect("Could not find table");
        table.rows.clear();
        table.row_count = 0;
    }

    pub fn get_table(&mut self, name: String) -> Option<&Box<Table>> {
        if let Some(table) = self.tables.get_mut(&name) {
            return Some(table);
        }
        return None;
    }

    pub fn add_table_column(&mut self, name: String, column: Box<Column>) {
        let table = self.tables.get_mut(&name).expect("Could not find table");
        table.columns.push(column.clone());
        for (_, table) in &mut self.tables {
            for row in &mut table.rows {
                match column.data_type {
                    DataType::INT => {
                        row.push(Cell::INT(0, column.size, column.nullable));
                    },
                    DataType::STRING => {
                        row.push(Cell::STRING("".to_string(), column.size, column.nullable));
                    },
                    _ => panic!("Unsupported data type")
                }
            }
        }
    }

    pub fn describe_tables(&mut self) {
        let mut table_names: Vec<&String> = self.tables.keys().collect();
        table_names.sort();
        for name in table_names {
            let table = self.tables.get(name).unwrap();
            println!("Table name: {}", name);
            println!("\tRow count: {}", table.row_count);
            println!("\tColumn count: {}", table.columns.len());
        }
    }

    pub fn insert_into_table(&mut self, name: String, columns: Vec<String>, values: Vec<String>) {
        let table = self.tables.get_mut(&name).expect("Could not find table");
        let table_columns = &table.columns;

        // Check if all columns exist
        let mut columns_missing = 0;
        for column in &columns {
            let c = table.find_column(column.clone());
            if c.is_none() {
                columns_missing += 1;
                panic!("Column not found: {}", column);
            }
        }
        if columns_missing > 0 {
            panic!("One or more columns do not exist in the table");
        }

        // Insert row
        let mut row: Vec<Cell> = Vec::new();
        for column in table_columns {
            for i in 0..columns.len() {
                if columns[i] == column.name {
                    
                    let mut value = values[i].clone();
                    match column.data_type {
                        DataType::INT => {
                            let value = value.parse::<i32>().expect("Could not parse integer");
                            row.push(Cell::INT(value, column.size, column.nullable));
                        },
                        DataType::STRING => {
                            if (value.len() as u32) > column.size {
                                value = value.chars().take(column.size as usize).collect();
                            }
                            row.push(Cell::STRING(value, column.size, column.nullable));
                        },
                        _ => panic!("Unsupported data type")
                    }
                }
            }
        }
        table.insert_row(row);
        self.tables.get_mut(&name).unwrap().row_count += 1;
    }

    pub fn select_from_table(&mut self, name: String, distinct: bool, columns: Vec<String>) {
        let table = self.tables.get(&name).expect("Could not find table");
        let table_columns = &table.columns;

        // Check if all columns exist
        let mut columns_missing = 0;
        for column in &columns {
            if column != "*" {
                let c = table.find_column(column.clone());
                if c.is_none() {
                    columns_missing += 1;
                    panic!("Column not found: {}", column);
                }
            }
        }
        if columns_missing > 0 {
            panic!("One or more columns do not exist in the table");
        }

        println!("Results:");

        let mut outputColumns = Vec::new();
        let mut outputRows = Vec::new();

        // Get column indexes
        let mut column_indexes: Vec<usize> = Vec::new();
        for column in &columns {
            if column == "*" {
                for (index, table_column) in table_columns.iter().enumerate() {
                    column_indexes.push(index);
                    //print!(" {:<15} |", table_column.name);
                    outputColumns.push(table_column.name.clone());
                }
            }
            else {
                for (index, table_column) in table_columns.iter().enumerate() {
                    if *column == table_column.name {
                        column_indexes.push(index);
                        //print!(" {:<15} |", table_column.name);
                        outputColumns.push(table_column.name.clone());
                    }
                }
            }
        }
        println!();

        for row in &table.rows {
            let mut outputRow: Vec<Cell> = Vec::new();
            for index in &column_indexes {
                for i in 0..row.len() {
                    if i == *index {
                        match &row[i] {
                            Cell::INT(value, _, _) => {
                                //print!(" {:<15} |", value);
                                outputRow.push(Cell::INT(*value, 0, false));
                            },
                            Cell::STRING(value, _, _) => {
                                //print!(" {:<15} |", value);
                                outputRow.push(Cell::STRING(value.clone(), 0, false));
                            }
                        }
                    }
                }
            }
            //println!();
            outputRows.push(outputRow);
        }

        for column in outputColumns {
            print!(" {:<15} |", column);
        }
        println!();

        if distinct {
            let mut distinctRows: Vec<Vec<Cell>> = Vec::new();
            for outputRow in outputRows {
                let mut unique = true;
                for i in 0..outputRow.len() {
                    for distinctRow in &distinctRows {
                        if self.compareEqCells(&outputRow[i], &distinctRow[i]) {
                            unique = false;
                        }
                    }
                }
                if unique {
                    distinctRows.push(outputRow.clone());
                }
            }
            outputRows = distinctRows;
        }

        for row in outputRows {
            for cell in row {
                match cell {
                    Cell::INT(value, _, _) => {
                        print!(" {:<15} |", value);
                    },
                    Cell::STRING(value, _, _) => {
                        print!(" {:<15} |", value);
                    }
                }
            }
            println!();
        }

        println!();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut database = Database {
        tables: HashMap::new(),
    };
    let file_path: &String = &args[1];
    let content = fs::read_to_string(file_path).expect("Cannot read file");
    let mut parser = Box::new(Parser {
        cursor: 0,
        tokens: Vec::new()
    });
    let top = parser.generate_ast(content.clone());
    let mut stack: Vec<Option<Box<ASTNode>>> = Vec::new();
    stack.push(top);
    //println!("{:?}", top);
    while !stack.is_empty() {
        let node = stack.pop();
        match node {
            Some(node) => {
                
                match *node.unwrap() {
                    ASTNode::CreateStatement { table_name, columns_to_add, next } => {
                        println!("Command: Create statement");
                        if database.get_table(table_name.clone()).is_some() {
                            panic!("Table already exists");
                        }
                        database.create_table(table_name.clone());
                        for column in columns_to_add {
                            database.add_table_column(table_name.clone(), Box::new(column.clone()));
                        }
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                    ASTNode::DropStatement { table_name, next } => {
                        println!("Command: Drop statement");
                        if database.get_table(table_name.clone()).is_none() {
                            panic!("Table does already exists");
                        }
                        database.drop_table(table_name.clone());
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                    ASTNode::SelectStatement { table_name, distinct, columns, next } => {
                        println!("Command: Select statement");
                        database.select_from_table(table_name.clone(), distinct, columns);
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                    ASTNode::InsertStatement { table_name, columns, values, next } => {
                        println!("Command: Insert statement");
                        database.insert_into_table(table_name.clone(), columns, values);
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                    ASTNode::ShowTablesStatement { next } => {
                        println!("Command: Show tables statement");
                        database.describe_tables();
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                    ASTNode::AlterTableStatement { table_name, columns_to_add, next } => {
                        println!("Command: Alter statement");
                        for column in columns_to_add {
                            database.add_table_column(table_name.clone(), Box::new(column.clone()));
                        }
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                    ASTNode::TruncateTableStatement { table_name, next } => {
                        println!("Command: Truncate statement");
                        database.truncate_table(table_name.clone());
                        stack.pop();
                        if next.is_some() {
                            stack.push(next);
                        }
                    },
                }
            },
            None => {
                break;
            }
        }
    }
}
