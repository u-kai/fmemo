use crate::schema::{Level, Memo, MemoBuilder, CodeBlock};

pub fn parse_memo(content: &str) -> Vec<Memo> {
    let flat_memos = parse_flat(content);
    build_hierarchy(flat_memos)
}

fn parse_flat(content: &str) -> Vec<Memo> {
    let mut memos = Vec::new();
    let mut current_memo: Option<MemoBuilder> = None;
    let mut in_code_block = false;
    let mut current_code = String::new();
    let mut current_lang = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                if let Some(ref mut builder) = current_memo {
                    *builder = builder.clone().add_code_block(current_lang.clone(), current_code.trim().to_string());
                }
                current_code.clear();
                current_lang.clear();
                in_code_block = false;
            } else {
                // Start of code block
                current_lang = line[3..].to_string();
                in_code_block = true;
            }
        } else if in_code_block {
            current_code.push_str(line);
            current_code.push('\n');
        } else if line.starts_with('#') {
            // Save current memo before creating new one
            if let Some(builder) = current_memo.take() {
                let (final_content, description) = extract_description(&current_content);
                let mut final_builder = builder.content(final_content.trim().to_string());
                if let Some(desc) = description {
                    final_builder = final_builder.description(desc);
                }
                memos.push(final_builder.build());
            }
            
            let level_count = line.chars().take_while(|&c| c == '#').count() as u8;
            let title = line[level_count as usize..].trim().to_string();
            let level = Level::new(level_count - 1); // 0-indexed
            
            current_memo = Some(MemoBuilder::new(level, title));
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    
    // Handle the last memo
    if let Some(builder) = current_memo {
        let (final_content, description) = extract_description(&current_content);
        let mut final_builder = builder.content(final_content.trim().to_string());
        if let Some(desc) = description {
            final_builder = final_builder.description(desc);
        }
        memos.push(final_builder.build());
    }
    
    memos
}

fn extract_description(content: &str) -> (String, Option<String>) {
    // Look for <desc>...</desc> pattern
    let desc_start = "<desc>";
    let desc_end = "</desc>";
    
    if let Some(start_pos) = content.find(desc_start) {
        if let Some(end_pos) = content[start_pos..].find(desc_end) {
            let desc_content_start = start_pos + desc_start.len();
            let desc_content_end = start_pos + end_pos;
            
            let description = content[desc_content_start..desc_content_end].to_string();
            
            // Remove the desc tag from content
            let before_desc = &content[..start_pos];
            let after_desc = &content[start_pos + end_pos + desc_end.len()..];
            let cleaned_content = format!("{}{}", before_desc, after_desc);
            
            return (cleaned_content, Some(description));
        }
    }
    
    (content.to_string(), None)
}

fn build_hierarchy(flat_memos: Vec<Memo>) -> Vec<Memo> {
    let mut root_memos = Vec::new();
    let mut stack: Vec<MemoBuilder> = Vec::new();

    for memo in flat_memos {
        // Convert memo back to builder for hierarchy building
        let memo_level = memo.level().clone();
        let mut builder = MemoBuilder::new(memo_level.clone(), memo.title().clone());
        
        if let Some(content) = memo.content() {
            builder = builder.content(content.clone());
        }
        
        if let Some(description) = memo.description() {
            builder = builder.description(description.clone());
        }
        
        for code_block in memo.code_blocks() {
            builder = builder.add_code_block(code_block.language.clone(), code_block.code.clone());
        }

        // Pop stack until we find a parent or reach the root
        while let Some(last) = stack.last() {
            if last.level().level() < memo_level.level() {
                break;
            }
            let completed = stack.pop().unwrap();
            if let Some(parent) = stack.last_mut() {
                *parent = parent.clone().add_child(completed.build());
            } else {
                root_memos.push(completed.build());
            }
        }

        stack.push(builder);
    }

    // Process remaining items in stack
    while let Some(builder) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            *parent = parent.clone().add_child(builder.build());
        } else {
            root_memos.push(builder.build());
        }
    }

    root_memos
}

#[cfg(test)]
mod tests {
    use crate::schema::{MemoBuilder, Level};
    use super::parse_memo;

    #[test]
    fn test_simple_hierarchy() {
        let content = r#"
# Title
hoge

## Child
choge
"#;
        let result = parse_memo(content);
        let root_level = Level::root();
        let child = MemoBuilder::new(root_level.child(), "Child".to_string())
            .content("choge".to_string())
            .build();
        let expected = vec![MemoBuilder::new(root_level, "Title".to_string())
            .content("hoge".to_string())
            .add_child(child)
            .build()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_siblings() {
        let content = r#"
# First Section
content1

# Second Section
content2
"#;
        let result = parse_memo(content);
        let root_level = Level::root();
        let expected = vec![
            MemoBuilder::new(root_level.clone(), "First Section".to_string())
                .content("content1".to_string())
                .build(),
            MemoBuilder::new(root_level, "Second Section".to_string())
                .content("content2".to_string())
                .build(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deep_nesting() {
        let content = r#"
# Level 1
content level 1

## Level 2
content level 2

### Level 3
content level 3

#### Level 4
content level 4
"#;
        let result = parse_memo(content);
        let level1 = Level::root();
        let level2 = level1.child();
        let level3 = level2.child();
        let level4 = level3.child();
        
        let memo_l4 = MemoBuilder::new(level4, "Level 4".to_string())
            .content("content level 4".to_string())
            .build();
        let memo_l3 = MemoBuilder::new(level3, "Level 3".to_string())
            .content("content level 3".to_string())
            .add_child(memo_l4)
            .build();
        let memo_l2 = MemoBuilder::new(level2, "Level 2".to_string())
            .content("content level 2".to_string())
            .add_child(memo_l3)
            .build();
        let expected = vec![
            MemoBuilder::new(level1, "Level 1".to_string())
                .content("content level 1".to_string())
                .add_child(memo_l2)
                .build()
        ];
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_code_blocks() {
        let content = r#"
# Function Example
This is a function:

```rust
fn hello() {
    println!("Hello, world!");
}
```

More content here.

```python
def greet():
    print("Hello")
```
"#;
        let result = parse_memo(content);
        
        // Just verify the structure exists and has code blocks
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Function Example");
        assert_eq!(memo.code_blocks().len(), 2);
        assert_eq!(memo.code_blocks()[0].language, "rust");
        assert_eq!(memo.code_blocks()[1].language, "python");
        assert!(memo.content().as_ref().unwrap().contains("This is a function"));
    }

    #[test]
    fn test_mixed_content() {
        let content = r#"
# Main Section
Introduction text

```javascript
console.log("test");
```

## Subsection 1
Some content

### Details
Detailed info

## Subsection 2
More content here

```bash
ls -la
```
"#;
        let result = parse_memo(content);
        
        // Check basic structure
        assert_eq!(result.len(), 1);
        let main_section = &result[0];
        assert_eq!(main_section.title(), "Main Section");
        assert_eq!(main_section.children().len(), 2);
        assert_eq!(main_section.code_blocks().len(), 1);
        assert_eq!(main_section.code_blocks()[0].language, "javascript");
        
        // Check subsections
        let subsection1 = &main_section.children()[0];
        assert_eq!(subsection1.title(), "Subsection 1");
        assert_eq!(subsection1.children().len(), 1);
        assert_eq!(subsection1.children()[0].title(), "Details");
        
        let subsection2 = &main_section.children()[1];
        assert_eq!(subsection2.title(), "Subsection 2");
        assert_eq!(subsection2.code_blocks().len(), 1);
        assert_eq!(subsection2.code_blocks()[0].language, "bash");
    }

    #[test]
    fn test_empty_sections() {
        let content = r#"
# Empty Title

## Another Empty

### With Content
Some actual content
"#;
        let result = parse_memo(content);
        let level1 = Level::root();
        let level2 = level1.child();
        let level3 = level2.child();
        
        let with_content = MemoBuilder::new(level3, "With Content".to_string())
            .content("Some actual content".to_string())
            .build();
        
        let another_empty = MemoBuilder::new(level2, "Another Empty".to_string())
            .content("".to_string())
            .add_child(with_content)
            .build();
        
        let expected = vec![
            MemoBuilder::new(level1, "Empty Title".to_string())
                .content("".to_string())
                .add_child(another_empty)
                .build()
        ];
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_content_headers_only() {
        let content = r#"
# Title 1
# Title 2
## Subtitle
"#;
        let result = parse_memo(content);
        let level1 = Level::root();
        let level2 = level1.child();
        
        let subtitle = MemoBuilder::new(level2, "Subtitle".to_string())
            .content("".to_string())
            .build();
        
        let expected = vec![
            MemoBuilder::new(level1.clone(), "Title 1".to_string())
                .content("".to_string())
                .build(),
            MemoBuilder::new(level1, "Title 2".to_string())
                .content("".to_string())
                .add_child(subtitle)
                .build()
        ];
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_skipped_levels() {
        let content = r#"
# Level 1
content 1

#### Level 4 (skipped 2 and 3)
content 4

## Level 2
content 2
"#;
        let result = parse_memo(content);
        let level1 = Level::root();
        let level2 = level1.child();
        let level4 = Level::new(3); // 0-indexed, so level 4 is index 3
        
        let level4_memo = MemoBuilder::new(level4, "Level 4 (skipped 2 and 3)".to_string())
            .content("content 4".to_string())
            .build();
        
        let level2_memo = MemoBuilder::new(level2, "Level 2".to_string())
            .content("content 2".to_string())
            .build();
        
        let expected = vec![
            MemoBuilder::new(level1, "Level 1".to_string())
                .content("content 1".to_string())
                .add_child(level4_memo)
                .add_child(level2_memo)
                .build()
        ];
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let content = "";
        let result = parse_memo(content);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_only_whitespace() {
        let content = "   \n\n  \n   ";
        let result = parse_memo(content);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_single_header_no_content() {
        let content = "# Header Only";
        let result = parse_memo(content);
        let expected = vec![
            MemoBuilder::new(Level::root(), "Header Only".to_string())
                .content("".to_string())
                .build()
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_code_block_without_language() {
        let content = r#"
# Code Example

```
some code here
```
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Code Example");
        assert_eq!(memo.code_blocks().len(), 1);
        assert_eq!(memo.code_blocks()[0].language, "");
        assert_eq!(memo.code_blocks()[0].code, "some code here");
    }

    #[test]
    fn test_nested_code_blocks() {
        let content = r#"
# Parent

```rust
fn main() {}
```

## Child

```python
print("hello")
```
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let parent = &result[0];
        assert_eq!(parent.code_blocks().len(), 1);
        assert_eq!(parent.children().len(), 1);
        
        let child = &parent.children()[0];
        assert_eq!(child.code_blocks().len(), 1);
        assert_eq!(child.code_blocks()[0].language, "python");
    }

    #[test]
    fn test_multiple_code_blocks_in_section() {
        let content = r#"
# Multiple Codes

```rust
let x = 5;
```

Some text in between.

```javascript
let y = 10;
```

```python
z = 15
```
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.code_blocks().len(), 3);
        assert_eq!(memo.code_blocks()[0].language, "rust");
        assert_eq!(memo.code_blocks()[1].language, "javascript");
        assert_eq!(memo.code_blocks()[2].language, "python");
    }

    #[test]
    fn test_six_level_nesting() {
        let content = r#"
# Level 1
## Level 2
### Level 3
#### Level 4
##### Level 5
###### Level 6
Deep content
"#;
        let result = parse_memo(content);
        
        // Navigate down the nested structure
        let mut current = &result[0];
        for expected_level in 0..5 {
            assert_eq!(current.level().level(), expected_level);
            if expected_level < 5 {
                assert_eq!(current.children().len(), 1);
                current = &current.children()[0];
            }
        }
        
        // Final level should have the content
        assert_eq!(current.content().as_ref().unwrap(), "Deep content");
    }

    #[test]
    fn test_desc_tag_simple() {
        let content = r#"
# Function Name
<desc>This function calculates the sum of two numbers</desc>
Regular content here.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Function Name");
        assert_eq!(memo.description(), &Some("This function calculates the sum of two numbers".to_string()));
        assert!(memo.content().as_ref().unwrap().contains("Regular content here"));
    }

    #[test]
    fn test_desc_tag_with_code_blocks() {
        let content = r#"
# API Endpoint
<desc>Handles user authentication and returns JWT token</desc>

```rust
fn authenticate(user: &str, pass: &str) -> Result<String, Error> {
    // implementation here
}
```

Additional documentation here.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "API Endpoint");
        assert_eq!(memo.description(), &Some("Handles user authentication and returns JWT token".to_string()));
        assert_eq!(memo.code_blocks().len(), 1);
        assert_eq!(memo.code_blocks()[0].language, "rust");
        assert!(memo.content().as_ref().unwrap().contains("Additional documentation"));
    }

    #[test]
    fn test_desc_tag_nested_sections() {
        let content = r#"
# Main Module
<desc>Main application module</desc>

## Helper Functions
<desc>Utility functions for data processing</desc>

### Calculate Sum
<desc>Adds two integers</desc>
Implementation details here.

### Calculate Product
<desc>Multiplies two integers</desc>
More implementation details.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let main_module = &result[0];
        assert_eq!(main_module.title(), "Main Module");
        assert_eq!(main_module.description(), &Some("Main application module".to_string()));
        assert_eq!(main_module.children().len(), 1);
        
        let helper_functions = &main_module.children()[0];
        assert_eq!(helper_functions.title(), "Helper Functions");
        assert_eq!(helper_functions.description(), &Some("Utility functions for data processing".to_string()));
        assert_eq!(helper_functions.children().len(), 2);
        
        let calculate_sum = &helper_functions.children()[0];
        assert_eq!(calculate_sum.title(), "Calculate Sum");
        assert_eq!(calculate_sum.description(), &Some("Adds two integers".to_string()));
        assert!(calculate_sum.content().as_ref().unwrap().contains("Implementation details here"));
        
        let calculate_product = &helper_functions.children()[1];
        assert_eq!(calculate_product.title(), "Calculate Product");
        assert_eq!(calculate_product.description(), &Some("Multiplies two integers".to_string()));
        assert!(calculate_product.content().as_ref().unwrap().contains("More implementation details"));
    }

    #[test]
    fn test_desc_tag_multiline() {
        let content = r#"
# Complex Function
<desc>This is a multi-line description
that spans across multiple lines
and provides detailed information</desc>

Function implementation here.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Complex Function");
        let expected_desc = "This is a multi-line description\nthat spans across multiple lines\nand provides detailed information";
        assert_eq!(memo.description(), &Some(expected_desc.to_string()));
    }

    #[test]
    fn test_desc_tag_with_html_entities() {
        let content = r#"
# HTML Parser
<desc>Parses HTML content and handles &lt;tags&gt; and &amp; entities</desc>
Implementation details.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "HTML Parser");
        assert_eq!(memo.description(), &Some("Parses HTML content and handles &lt;tags&gt; and &amp; entities".to_string()));
    }

    #[test]
    fn test_desc_tag_empty() {
        let content = r#"
# Empty Description
<desc></desc>
Some content here.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Empty Description");
        assert_eq!(memo.description(), &Some("".to_string()));
    }

    #[test]
    fn test_no_desc_tag() {
        let content = r#"
# No Description
Just regular content without description tag.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "No Description");
        assert_eq!(memo.description(), &None);
        assert!(memo.content().as_ref().unwrap().contains("Just regular content"));
    }

    #[test]
    fn test_desc_tag_mixed_with_code() {
        let content = r#"
# Database Connection
<desc>Establishes connection to PostgreSQL database</desc>

```sql
SELECT * FROM users WHERE active = true;
```

<desc>Additional description after code</desc>

More content here.

```rust
fn connect() -> Connection {
    // implementation
}
```
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Database Connection");
        // Should capture the first desc tag
        assert_eq!(memo.description(), &Some("Establishes connection to PostgreSQL database".to_string()));
        assert_eq!(memo.code_blocks().len(), 2);
        assert_eq!(memo.code_blocks()[0].language, "sql");
        assert_eq!(memo.code_blocks()[1].language, "rust");
    }

    #[test]
    fn test_desc_tag_malformed() {
        let content = r#"
# Malformed Test
<desc>Missing closing tag
Regular content here.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Malformed Test");
        // Should treat it as regular content since no closing tag
        assert_eq!(memo.description(), &None);
        assert!(memo.content().as_ref().unwrap().contains("<desc>Missing closing tag"));
    }

    #[test]
    fn test_desc_tag_nested_tags() {
        let content = r#"
# Nested Tags
<desc>Description with <code>nested tags</code> inside</desc>
Regular content.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Nested Tags");
        assert_eq!(memo.description(), &Some("Description with <code>nested tags</code> inside".to_string()));
        assert!(memo.content().as_ref().unwrap().contains("Regular content"));
    }

    #[test]
    fn test_desc_tag_at_end() {
        let content = r#"
# End Description
Regular content first.

<desc>Description at the end</desc>
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "End Description");
        assert_eq!(memo.description(), &Some("Description at the end".to_string()));
        assert!(memo.content().as_ref().unwrap().contains("Regular content first"));
        assert!(!memo.content().as_ref().unwrap().contains("<desc>"));
    }

    #[test]
    fn test_multiple_desc_tags() {
        let content = r#"
# Multiple Descriptions
<desc>First description</desc>
Some content.
<desc>Second description</desc>
More content.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Multiple Descriptions");
        // Should only capture the first desc tag
        assert_eq!(memo.description(), &Some("First description".to_string()));
        assert!(memo.content().as_ref().unwrap().contains("Some content"));
        assert!(memo.content().as_ref().unwrap().contains("Second description"));
    }

    #[test]
    fn test_desc_tag_across_lines() {
        let content = r#"
# Cross Line Description
<desc>This description
spans multiple
lines with different formatting
and should be preserved</desc>

Implementation here.
"#;
        let result = parse_memo(content);
        assert_eq!(result.len(), 1);
        let memo = &result[0];
        assert_eq!(memo.title(), "Cross Line Description");
        let expected_desc = "This description\nspans multiple\nlines with different formatting\nand should be preserved";
        assert_eq!(memo.description(), &Some(expected_desc.to_string()));
        assert!(memo.content().as_ref().unwrap().contains("Implementation here"));
    }
}
