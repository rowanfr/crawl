use std::fmt;

use url::Url;

#[derive(Debug, PartialEq, Clone)]
pub struct SiteTree {
    pub current_site: Url,
    pub sub_sites: SubSites,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubSites {
    List(Vec<SiteTree>),
    Nil,
}

const TREE_ARRAY: [char; 4] = ['│', '├', '└', '─']; //Organizeed as Down, Down+Right, End, Right

impl fmt::Display for SiteTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut last: Vec<bool> = Vec::new();
        let mut output_string = String::new();
        self.print_recursive(0, 2, &mut last, &mut output_string);
        write!(f, "{}", output_string)
    }
}

impl SiteTree {
    #[allow(dead_code)]
    pub fn print(&self, spacing: usize) {
        let mut last: Vec<bool> = Vec::new();
        let mut output_string = String::new();
        self.print_recursive(0, spacing, &mut last, &mut output_string);
        print!("{}", output_string);
    }

    #[allow(dead_code)]
    pub fn print_to_string(&self, spacing: usize) -> String {
        let mut last: Vec<bool> = Vec::new();
        let mut output_string = String::new();
        self.print_recursive(0, spacing, &mut last, &mut output_string);
        output_string.to_string()
    }

    #[allow(dead_code)]
    pub fn as_ptr(&self) -> *const SiteTree {
        self as *const SiteTree
    }

    fn print_recursive(
        &self,
        mut depth: usize,
        spacing: usize,
        last: &mut Vec<bool>,
        output_string: &mut String,
    ) {
        let mut buffer_string = String::new();
        //This is an area where clippy is incorrect as this range loop is necessary to iterate to a specified depth not related to the index
        //This can be optimized further by using pointer dereferencing to avoid safety checks, but those checks are ultimatly trivial to perform and helpful overall
        #[allow(clippy::needless_range_loop)]
        for index in 0..depth {
            if depth - 1 == index {
                //String buffer with parent directly above
                if last[index] {
                    buffer_string.push(TREE_ARRAY[2])
                } else {
                    buffer_string.push(TREE_ARRAY[1])
                }
                let mut n = 0;
                while n < spacing {
                    buffer_string.push(TREE_ARRAY[3]);
                    n += 1;
                }
            } else {
                //String buffer with parent not directly above
                if last[index] {
                    buffer_string.push(' ')
                } else {
                    buffer_string.push(TREE_ARRAY[0])
                }
                let mut n = 0;
                while n < spacing {
                    buffer_string.push(' ');
                    n += 1;
                }
            }
        }

        output_string.push_str(buffer_string.as_str());
        output_string.push_str(self.current_site.as_str());
        output_string.push('\n');

        match &self.sub_sites {
            SubSites::List(sub_sites) => {
                let length = sub_sites.len();

                for (index, sub_site) in sub_sites.iter().enumerate() {
                    if (length - 1) == index {
                        if last.len() <= depth {
                            last.push(true);
                        } else {
                            last[depth] = true
                        }
                        depth += 1;
                        sub_site.print_recursive(depth, spacing, last, output_string);
                        depth -= 1;
                    } else {
                        if last.len() <= depth {
                            last.push(false);
                        } else {
                            last[depth] = false
                        }
                        depth += 1;
                        sub_site.print_recursive(depth, spacing, last, output_string);
                        depth -= 1;
                    }
                }
            }
            SubSites::Nil => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SiteTree, SubSites};
    use url::Url;

    #[test]
    fn test_tree_print() {
        let site_tree = SiteTree {
            current_site: Url::parse("https://example.com").unwrap(),
            sub_sites: SubSites::List(vec![
                SiteTree {
                    current_site: Url::parse("https://example.com/subsite1").unwrap(),
                    sub_sites: SubSites::Nil,
                },
                SiteTree {
                    current_site: Url::parse("https://example.com/subsite2").unwrap(),
                    sub_sites: SubSites::List(vec![
                        SiteTree {
                            current_site: Url::parse("https://example.com/subsite2/1").unwrap(),
                            sub_sites: SubSites::List(vec![SiteTree {
                                current_site: Url::parse("https://example.com/subsite2/1/1")
                                    .unwrap(),
                                sub_sites: SubSites::Nil,
                            }]),
                        },
                        SiteTree {
                            current_site: Url::parse("https://example.com/subsite2/2").unwrap(),
                            sub_sites: SubSites::List(vec![
                                SiteTree {
                                    current_site: Url::parse("https://example.com/subsite2/2/1")
                                        .unwrap(),
                                    sub_sites: SubSites::Nil,
                                },
                                SiteTree {
                                    current_site: Url::parse("https://example.com/subsite2/2/2")
                                        .unwrap(),
                                    sub_sites: SubSites::Nil,
                                },
                            ]),
                        },
                    ]),
                },
                SiteTree {
                    current_site: Url::parse("https://example.com/subsite3").unwrap(),
                    sub_sites: SubSites::Nil,
                },
            ]),
        };
        println!("Output of tree print:");
        println!("{}", site_tree);
        assert_eq!(
            format!("{}", site_tree),
            "https://example.com/\n├──https://example.com/subsite1\n├──https://example.com/subsite2\n│  ├──https://example.com/subsite2/1\n│  │  └──https://example.com/subsite2/1/1\n│  └──https://example.com/subsite2/2\n│     ├──https://example.com/subsite2/2/1\n│     └──https://example.com/subsite2/2/2\n└──https://example.com/subsite3\n"
        );
    }
}
