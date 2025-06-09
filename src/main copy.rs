use std::io::Write;
use fdt_parser::*;
use std::fs::File;
use vm_fdt::{FdtWriterNode, FdtWriter};

fn main() {
    println!("Hello, world!");

    const TEST_FDT: &[u8] = include_bytes!("../dtb/bcm2711-rpi-4-b.dtb");
    const TEST_PHYTIUM_FDT: &[u8] = include_bytes!("../dtb/phytium.dtb");
    const TEST_QEMU_FDT: &[u8] = include_bytes!("../dtb/qemu_pci.dtb");
    const TEST_3568_FDT: &[u8] = include_bytes!("../dtb/rk3568-firefly-roc-pc-se.dtb");

    let fdt = Fdt::from_bytes(TEST_3568_FDT).unwrap();

    // let new_reg: [u32; 4] = [0x1, 0x2, 0x3, 0x4];
    let mut new_fdt = FdtWriter::new().unwrap();
    let root_node = new_fdt.begin_node("").unwrap();
    let mut old_node_level = 1;
    let mut child_node: Vec<FdtWriterNode> = Vec::new();

    for node in fdt.all_nodes() {

        println!("node name: {:#?}, leval {}", node.name(), node.level);
        println!("node leval: {}, old leval {}", node.level, old_node_level);

        if node.level == 1 {
            for prop in node.propertys() {
                new_fdt.property(prop.name, prop.raw_value()).unwrap();
            }
            continue; // 跳过根节点
        }

        if node.level == old_node_level {
            println!("关闭节点: {:#?}, level:{}", node.name(), node.level);
            new_fdt.end_node(child_node.pop().unwrap()).unwrap();
        }
        if node.level < old_node_level {
            for _ in node.level..=old_node_level {
                // let end_node = child_node.pop().unwrap();
                println!("关闭节点: {}", node.level);
                new_fdt.end_node(child_node.pop().unwrap()).unwrap();
            }
        }
        old_node_level = node.level;

        child_node.push(new_fdt.begin_node(node.name()).unwrap());
        for prop in node.propertys() {
            println!("node prop: {:#?} - {:#?}", prop.name, prop.raw_value());
            new_fdt.property(prop.name, prop.raw_value()).unwrap();
        }

    }
    if child_node.len() > 0 {
        println!("关闭剩余节点: {:#?}", child_node.len());
    }
    while let Some(node) = child_node.pop() {
        println!("关闭节点: {:#?}", node);
        new_fdt.end_node(node).unwrap();
    }
    new_fdt.end_node(root_node).unwrap();

    let actual_new_fdt = new_fdt.finish().unwrap();
    let mut file = File::create("out.dtb").unwrap();
    file.write_all(&actual_new_fdt).unwrap();
    // println!("修改后的 FDT 大小: {:#?}", new_fdt);

    println!("world end");

    // const OUT: &[u8] = include_bytes!("../out.dtb");
    // compare_fdt(fdt, Fdt::from_bytes(OUT).unwrap());    
}

fn compare_fdt(fdt1: Fdt<'_>, fdt2: Fdt<'_>) -> bool {
    let stat1 = fdt1.find_nodes("/sata@fc000000").next().unwrap();
    println!("stat1: {:#?}", stat1.name());
    for prop in stat1.propertys() {
        println!("prop: {:#?} - {:#?}", prop.name, prop.raw_value());
    }
    let stat2 = fdt2.find_nodes("/sata@fc000000").next().unwrap();
    println!("stat2: {:#?}", stat2.name());
    for prop in stat2.propertys() {
        println!("prop: {:#?} - {:#?}", prop.name, prop.raw_value());
    }

    true
}