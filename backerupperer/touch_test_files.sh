#!/bin/bash
# Script to create (touch) all files listed in the test_files directory

TEST_DIR="$(dirname "$0")/test_files"

# Create the test_files directory if it doesn't exist
mkdir -p "$TEST_DIR"

# List of files to create (edit this list as needed)
FILES=(
vzdump-lxc-1301-2025_08_24-01_51_39.log		  vzdump-qemu-401-2025_08_01-18_42_16.vma.dat
vzdump-lxc-1301-2025_08_24-01_51_39.tar.gz	  vzdump-qemu-401-2025_08_01-18_46_32.log
vzdump-lxc-1301-2025_08_24-01_51_39.tar.gz.notes  vzdump-qemu-401-2025_08_01-18_46_51.log
vzdump-lxc-1401-2025_08_24-01_52_32.log		  vzdump-qemu-401-2025_08_01-18_51_26.tmp
vzdump-lxc-1401-2025_08_24-01_52_32.tar.gz	  vzdump-qemu-401-2025_08_01-18_51_26.vma.dat
vzdump-lxc-1401-2025_08_24-01_52_32.tar.gz.notes  vzdump-qemu-402-2025_08_27-22_43_30.log
vzdump-qemu-200-2025_08_24-01_00_04.log		  vzdump-qemu-402-2025_08_27-22_43_30.vma.gz
vzdump-qemu-200-2025_08_24-01_00_04.vma.gz	  vzdump-qemu-402-2025_08_27-22_43_30.vma.gz.notes
vzdump-qemu-200-2025_08_24-01_00_04.vma.gz.notes  vzdump-qemu-402-2025_08_28-02_43_34.log
vzdump-qemu-301-2025_08_27-22_30_00.log		  vzdump-qemu-402-2025_08_28-02_43_34.vma.gz
vzdump-qemu-301-2025_08_27-22_30_00.vma.gz	  vzdump-qemu-402-2025_08_28-02_43_34.vma.gz.notes
vzdump-qemu-301-2025_08_27-22_30_00.vma.gz.notes  vzdump-qemu-402-2025_08_28-22_43_57.log
vzdump-qemu-301-2025_08_28-02_30_01.log		  vzdump-qemu-402-2025_08_28-22_43_57.vma.gz
vzdump-qemu-301-2025_08_28-02_30_01.vma.gz	  vzdump-qemu-402-2025_08_28-22_43_57.vma.gz.notes
vzdump-qemu-301-2025_08_28-02_30_01.vma.gz.notes  vzdump-qemu-402-2025_08_29-02_43_42.log
vzdump-qemu-301-2025_08_28-22_30_03.log		  vzdump-qemu-402-2025_08_29-02_43_42.vma.gz
vzdump-qemu-301-2025_08_28-22_30_03.vma.gz	  vzdump-qemu-402-2025_08_29-02_43_42.vma.gz.notes
vzdump-qemu-301-2025_08_28-22_30_03.vma.gz.notes  vzdump-qemu-501-2025_08_24-01_24_32.log
vzdump-qemu-301-2025_08_29-02_30_03.log		  vzdump-qemu-501-2025_08_24-01_24_32.vma.gz
vzdump-qemu-301-2025_08_29-02_30_03.vma.gz	  vzdump-qemu-501-2025_08_24-01_24_32.vma.gz.notes
vzdump-qemu-301-2025_08_29-02_30_03.vma.gz.notes  vzdump-qemu-601-2025_08_24-01_32_31.log
vzdump-qemu-401-2025_07_22-20_32_56.log		  vzdump-qemu-601-2025_08_24-01_32_31.vma.gz
vzdump-qemu-401-2025_08_01-18_42_16.tmp		  vzdump-qemu-601-2025_08_24-01_32_31.vma.gz.notes
)

for filename in "${FILES[@]}"; do
    touch "$TEST_DIR/$filename"
done

echo "Created files in $TEST_DIR: ${FILES[*]}"
