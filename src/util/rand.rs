pub fn choice<'a, T>(choices: &'a [T], weights: &[u32]) -> &'a T {
    assert!(!choices.is_empty());
    assert_eq!(choices.len(), weights.len());

    let (sum, cumsum) = weights.iter().fold(
        (0, Vec::with_capacity(choices.len())),
        |(mut sum, mut cumsum), &w| {
            sum += w;
            cumsum.push(sum);

            (sum, cumsum)
        },
    );

    let t = rand::random_range(0..sum);

    let i = cumsum.iter().position(|&x| x > t).unwrap();

    &choices[i]
}
