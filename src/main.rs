use fdt_parser::*;
use std::fs;
use std::fs::File;
use std::{
    io::Write,
    os::unix::fs::MetadataExt,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use vm_fdt::{FdtWriter, FdtWriterNode};

fn main() {
    println!("Hello, world!");

    const TEST_FDT: &[u8] = include_bytes!("../dtb/bcm2711-rpi-4-b.dtb");
    const TEST_PHYTIUM_FDT: &[u8] = include_bytes!("../dtb/phytium.dtb");
    const TEST_QEMU_FDT: &[u8] = include_bytes!("../dtb/qemu_pci.dtb");
    const TEST_3568_FDT: &[u8] = include_bytes!("../dtb/rk3568-firefly-roc-pc-se.dtb");

    let fdt = Fdt::from_bytes(TEST_3568_FDT).unwrap();

    let new_reg: [u32; 4] = [0x1, 0x2, 0x3, 0x6];

    /*
    let mut new_fdt = FdtWriter::new().unwrap();
    let mut old_node_level = 0;
    let mut child_node: Vec<FdtWriterNode> = Vec::new();

    for node in fdt.all_nodes() {

        println!("node name: {:#?}, leval {}", node.name(), node.level);
        println!("node leval: {}, old leval {}", node.level, old_node_level);

        if node.level <= old_node_level {
            for _ in node.level..=old_node_level {
                let end_node = child_node.pop().unwrap();
                println!("close node: {:#?}", end_node);
                new_fdt.end_node(end_node).unwrap();
            }
        }
        old_node_level = node.level;

        if node.name() == "/" {
            child_node.push(new_fdt.begin_node("").unwrap());
        } else {
            child_node.push(new_fdt.begin_node(node.name()).unwrap());
        }

        for prop in node.propertys() {
            // println!("node prop: {:#?} - {:#?}", prop.name, prop.raw_value());
            new_fdt.property(prop.name, prop.raw_value()).unwrap();
        }
    }
    //关闭剩余节点
    while let Some(node) = child_node.pop() {
        // println!("close node: {:#?}", node);
        new_fdt.end_node(node).unwrap();
    }
    */

    let new_fdt = change_node_property(&fdt, "sata@fc000000", "reg", &new_reg);

    let actual_new_fdt = new_fdt.finish().unwrap();
    let mut file = File::create("out.dtb").unwrap();
    file.write_all(&actual_new_fdt).unwrap();
    // println!("修改后的 FDT 大小: {:#?}", new_fdt);

    println!("world end");
    // sleep(Duration::from_secs(120));
    print_file_meta("out.dtb");
    let out_bytes = fs::read("out.dtb").unwrap();
    // const OUT: &[u8] = include_bytes!("../out.dtb");
    println!(
        "compare result: {}",
        compare_fdt(fdt, Fdt::from_bytes(&out_bytes).unwrap())
    );
}

fn compare_fdt(fdt1: Fdt<'_>, fdt2: Fdt<'_>) -> bool {
    let fdt1_collection = fdt1.all_nodes().collect::<Vec<_>>();
    let fdt2_collection = fdt2.all_nodes().collect::<Vec<_>>();
    if fdt1_collection.len() != fdt2_collection.len() {
        println!(
            "FDT nodes count mismatch: {} vs {}",
            fdt1_collection.len(),
            fdt2_collection.len()
        );
        return false;
    }
    for (node1, node2) in fdt1_collection.iter().zip(fdt2_collection.iter()) {
        if node1.name() != node2.name() {
            println!("Node name mismatch: {} vs {}", node1.name(), node2.name());
            return false;
        }
        if node1.propertys().count() != node2.propertys().count() {
            println!(
                "Property count mismatch in node {}: {} vs {}",
                node1.name(),
                node1.propertys().count(),
                node2.propertys().count()
            );
            return false;
        }
        for (prop1, prop2) in node1.propertys().zip(node2.propertys()) {
            if node1.name() == "bt_pins" {
                println!(
                    "name: {} vs {}; vlaue: {:#?} vs {:#?}",
                    prop1.name,
                    prop1.name,
                    prop1.str(),
                    prop2.str()
                );
            }
            if prop1.name != prop2.name {
                println!("Property name mismatch: {} vs {}", prop1.name, prop2.name);
                return false;
            }
            if prop1.raw_value() != prop2.raw_value() {
                println!(
                    "Property value mismatch for {}: {:?} vs {:?}",
                    prop1.name,
                    prop1.raw_value(),
                    prop2.raw_value()
                );
                return false;
            }
        }
    }
    true
}

fn change_node_property(
    fdt: &Fdt<'_>,
    node_name: &str,
    prop_name: &str,
    new_value: &[u32],
) -> FdtWriter {
    let mut new_fdt = FdtWriter::new().unwrap();
    let mut old_node_level = 0;
    let mut child_node: Vec<FdtWriterNode> = Vec::new();

    for node in fdt.all_nodes() {
        if node.name() == node_name {
            println!("Changing property {} in node {}", prop_name, node.name());
        }

        if node.level <= old_node_level {
            for _ in node.level..=old_node_level {
                let end_node = child_node.pop().unwrap();
                new_fdt.end_node(end_node).unwrap();
            }
        }
        old_node_level = node.level;

        if node.name() == "/" {
            child_node.push(new_fdt.begin_node("").unwrap());
        } else {
            child_node.push(new_fdt.begin_node(node.name()).unwrap());
        }

        for prop in node.propertys() {
            if prop.name == prop_name && node.name() == node_name {
                println!("Updating property {} with new value", prop_name);
                // new_fdt.property(prop.name, new_value).unwrap();
                new_fdt.property_array_u32(prop.name, new_value).unwrap();
            } else {
                new_fdt.property(prop.name, prop.raw_value()).unwrap();
            }
        }
    }

    while let Some(node) = child_node.pop() {
        new_fdt.end_node(node).unwrap();
    }

    new_fdt
}

fn print_file_meta(path: &str) {
    println!("当前时间1: {:?}", SystemTime::now());
    let meta = fs::metadata(path).unwrap();
    println!("文件大小: {} 字节", meta.len());
    println!(
        "最后访问时间: {:?}",
        UNIX_EPOCH + std::time::Duration::from_secs(meta.atime() as u64)
    );
    println!(
        "最后修改时间: {:?}",
        UNIX_EPOCH + std::time::Duration::from_secs(meta.mtime() as u64)
    );
    println!(
        "inode 变更时间(ctime): {:?}",
        UNIX_EPOCH + std::time::Duration::from_secs(meta.ctime() as u64)
    );
    println!("当前时间2: {:?}", SystemTime::now());
}
