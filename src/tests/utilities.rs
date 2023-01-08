use crate::utilities::{unify_paths, xml_to_plain, xml_to_text};
use std::path::PathBuf;
use roxmltree::{Document, Node, NodeType};

#[test]
fn test_unify_paths_case_1() {
    let p1 = PathBuf::from("/home/user/documents");
    let p2 = PathBuf::from("../projects");
    let unified = unify_paths(p1, p2);
    assert_eq!(unified, PathBuf::from("/home/user/projects"));
}

#[test]
fn test_unify_paths_case_2() {
    let p1 = PathBuf::from("/home/user/documents");
    let p2 = PathBuf::from("../../projects");
    let unified = unify_paths(p1, p2);
    assert_eq!(unified, PathBuf::from("/home/projects"));
}

#[test]
fn test_unify_paths_case_3() {
    let p1 = PathBuf::from("/home/user/documents");
    let p2 = PathBuf::from("../../../../../projects");
    let unified = unify_paths(p1, p2);
    assert_eq!(unified, PathBuf::from("/projects"));
}

#[test]
fn test_unify_paths_case_4() {
    let p1 = PathBuf::from("/home/user/documents");
    let p2 = PathBuf::from("project1/project2/../project3");
    let unified = unify_paths(p1, p2);
    assert_eq!(
        unified,
        PathBuf::from("/home/user/documents/project1/project3")
    );
}

#[test]
fn test_unify_paths_case_5() {
    let p1 = PathBuf::from("/home/user/documents");
    let p2 = PathBuf::from("project1/./project2/project3");
    let unified = unify_paths(p1, p2);
    assert_eq!(
        unified,
        PathBuf::from("/home/user/documents/project1/project2/project3")
    );
}


#[test]
fn test_xml_to_text() {
    let xml = "<div>\
            <div/>\
            <body>\
                <h1>Heading 1</h1>\
                <p>Paragraph 1</p>\
                <p>Paragraph 2</p>\
            </body>\
        </div>";
    let expected_output = "Heading 1\nParagraph 1\nParagraph 2\n";
    assert_eq!(xml_to_text(xml), expected_output);
}

#[test]
fn test_xml_to_text_whitespace() {
    let xml = "<div>\
            <body>\
            <p> Paragraph 1 </p>\
            <p>Paragraph 2</p>\
            </body>\
        </div>";
    let expected_output = " Paragraph 1 \nParagraph 2\n";
    assert_eq!(xml_to_text(xml), expected_output);
}

#[test]
fn test_xml_to_text_br_element() {
    let xml = "<div>\
             <body>\
            <p>Paragraph 1</p>\
            <br/>\
            <p>Paragraph 2</p>\
            </body>\
        </div>";
    let expected_output = "Paragraph 1\n\nParagraph 2\n";
    assert_eq!(xml_to_text(xml), expected_output);
}

#[test]
fn test_xml_to_text_li_element() {
    let xml = "<div>\
        <ul>\
            <li>Item 1</li>\
            <li>Item 2</li>\
        </ul>\
        </div>";
    let expected_output = "- Item 1\n- Item 2\n";
    assert_eq!(xml_to_text(xml), expected_output);
}

#[test]
fn test_xml_to_text_nested_elements() {
    let xml = "\
        <div>\
        <body>\
            <h1>Heading 1</h1>\
            <p>Paragraph 1</p>\
            <blockquote>\
                <p>Blockquote paragraph 1</p>\
                <p>Blockquote paragraph 2</p>\
            </blockquote>\
            <p>Paragraph 2</p>\
        </body>\
        </div>\
    ";
    let expected_output = "Heading 1\nParagraph 1\nBlockquote paragraph 1\nBlockquote paragraph 2\n\nParagraph 2\n";
    assert_eq!(xml_to_text(xml), expected_output);
}

