
use std::fs::read;

pub fn get_mnist_images(image_path: &str, label_path: &str) -> Result<(Vec<Vec<u8>>, Vec<u8>), ()> {
    let raw_image_data = read(image_path).unwrap();
    let raw_label_data = read(label_path).unwrap();

    let (num_image_dims, image_dim_sizes) = parse_idx_meta(&raw_image_data);
    let (num_label_dims, label_dim_sizes) = parse_idx_meta(&raw_label_data);

    let mut images: Vec<Vec<u8>> = Vec::new();
    let mut labels: Vec<u8> = Vec::new();


    //println!("Image tensor dimensions: {:?}", image_dim_sizes);
    //println!("Label tensor dimensions: {:?}", label_dim_sizes);

    assert_eq!(image_dim_sizes[0], label_dim_sizes[0]);

    images.reserve(image_dim_sizes[0] as usize);
    labels.reserve(label_dim_sizes[0] as usize);

    let offset = 4 + num_image_dims as usize * 4;
    let size = image_dim_sizes[1] as usize * image_dim_sizes[2] as usize;

    for i in 0..image_dim_sizes[0] as usize {
        // get images
        let start = offset + i * size;
        let mut img: Vec<u8> = Vec::new();
        img.extend_from_slice(&raw_image_data[start..(start + size)]);
        images.push(img);

        // get labels
        labels.push(raw_label_data[4 * (1 + num_label_dims as usize) + i]);
    }
    
    Ok((images, labels))

}

// returns dimension count, dimension sizes.
fn parse_idx_meta(data: &Vec<u8>) -> (u8, Vec<u32>) {
    let dimension_count = data[3].clone();

    let mut dimension_sizes: Vec<u32> = Vec::new(); 
    dimension_sizes.reserve(dimension_count as usize);

    for i in 0..dimension_count as usize {
        dimension_sizes.push(u32::from_be_bytes(
                <[u8; 4]>::try_from(&data[4 * (i + 1)..4 * (2 + i)]).unwrap()));
    }
    (dimension_count, dimension_sizes)
}
