extern crate rustbox;
extern crate chrono;

use rustbox::*;
use chrono::*;

use screen::common::*;
use utility::string::*;
use model::ShowItem;
use model::ShowReplyItem;
use reply_model::*;

pub struct Show<'a> {
    rustbox: &'a rustbox::RustBox,
    scrollY: usize,
}

impl<'a> Show<'a> {
    pub fn new(rustbox: &'a rustbox::RustBox) -> Self {
        Show {
            rustbox: &rustbox,
            scrollY: 0,
        }
    }
    pub fn print(&mut self, title: &str, item: &ShowItem) {

        self.print_header(&format!("{} - {} [{}/{}]",
                                   item.title,
                                   title,
                                   item.page,
                                   item.max_page));
        self.print_body(2, &item);
    }

    fn print_separator_top(&mut self, reply: &ShowReplyItem, y: usize) {
        if y > self.scrollY + 1 {
            let (replier_name, time) = self.build_separator_content(&reply);
            self.rustbox.print(0,
                               y - self.scrollY,
                               rustbox::RB_NORMAL,
                               Color::Green,
                               Color::Black,
                               &self.build_separator_top(&replier_name, &time));
        }
    }

    fn print_separator_bottom(&mut self, y: usize) {
        if y > self.scrollY + 1 {
            self.rustbox.print(0,
                               y - self.scrollY,
                               rustbox::RB_NORMAL,
                               Color::Green,
                               Color::Black,
                               &self.build_separator_bottom());
        }
    }

    fn print_header(&mut self, text: &str) {
        let title_len = jks_len(text);
        let padding = (if self.rustbox.width() >= title_len {
            self.rustbox.width() - title_len
        } else {
            0
        }) / 2;

        let header_bottom = seq_str_gen(0, self.rustbox.width(), "─", "");

        clearline(&self.rustbox, self.rustbox.width(), 0, 0);
        self.rustbox.print(padding,
                           0,
                           rustbox::RB_BOLD,
                           Color::White,
                           Color::Black,
                           text);
        self.rustbox.print(0,
                           1,
                           rustbox::RB_BOLD,
                           Color::Yellow,
                           Color::Black,
                           &header_bottom);
    }

    pub fn print_body(&mut self, offset_y: usize, item: &ShowItem) {
        let width = self.body_width();
        let rows = self.body_height();
        let rustbox = self.rustbox;
        let scrollY = self.scrollY;

        let mut y = offset_y;

        for (i, reply) in item.replies.iter().take(rows).enumerate() {

            y += self.print_reply(&reply.body, 0, y);

            self.print_separator_top(&reply, y);
            y += 1;

            self.print_separator_bottom(y);
            y += 1;
        }
    }

    fn print_reply(&mut self, vec: &Vec<NodeType>,
                   depth: usize,
                   y: usize)
                   -> usize {

       let rustbox = self.rustbox;
       let scrollY = self.scrollY;

        let padding = seq_str_gen(0, depth, "├─", "");
        let mut m = 0;
        let mut recursive_offset = 0;
        let mut total_y = 0;
        let mut line = String::new();

        // clean up lines (end)
        let vec2 = {
            let vec_length = vec.len();
            let vec_check_cleanup = vec.clone();

            // check if last 4 elements match the EMPTY PATTERN
            let is_last4_empty = vec_check_cleanup.iter()
                                                  .rev()
                                                  .take(4)
                                                  .enumerate()
                                                  .all(|(j, node)| match node.clone() {
                                                      NodeType::Br(n) => j == 1 || j == 2 || j == 3,
                                                      NodeType::Text(n) => j == 0 && n.data.is_empty(),
                                                      _ => false,
                                                  });

            let vec_short_length = if vec_length > 4 && is_last4_empty {
                vec_length - 4
            } else {
                vec_length
            };

            vec.iter().take(vec_short_length)
        };

        // clean up lines (start)
        let vec3 = {
            let vec2_cloned = vec2.clone();
            let mut result: Vec<NodeType> = Vec::new();
            for (j, node) in vec2_cloned.enumerate() {
                let node2 = node.clone();
                let node3 = node.clone();
                match node2 {
                    NodeType::Br(n) => {
                        if !result.is_empty() {
                            result.push(node3);
                        }
                    }
                    _ => result.push(node3),
                }
            }
            result.clone()
        };

        let mut is_first = true;
        for (j, node) in vec3.iter().enumerate() {
            total_y = y + m + recursive_offset;
            if scrollY + 1 < total_y {
                let node2 = node.clone();
                match node2 {
                    NodeType::Text(n) => {
                        if n.data != "" {
                            line = format!("{}{}", line, n.data);
                        }
                    }
                    NodeType::Image(n) => {
                        if n.data != "" {
                            line = format!("{}[img {}]", line, n.data);
                        }
                    }
                    NodeType::BlockQuote(n) => {
                        recursive_offset += self.print_reply(&n.data, depth + 1, total_y);
                        is_first = false;
                    }
                    NodeType::Br(n) => {
                        if !line.is_empty() {
                            print_default(rustbox,
                                          0,
                                          total_y - scrollY,
                                          format!(" {}{}", padding, line));
                            line = String::new();
                            is_first = false;
                        }

                        // prevent first line empty
                        if !is_first {
                            m += 1;
                        }

                    }
                }
            }
        }

        if !line.is_empty() {
            total_y = y + m + recursive_offset;
            print_default(rustbox,
                          0,
                          total_y - scrollY,
                          format!(" {}{}  ", padding, line));
            line = String::new();
            m += 1;
        }

        m + recursive_offset
    }

    fn build_separator_content(&mut self, reply: &ShowReplyItem) -> (String, String) {
        let now = Local::now();

        let replier_name = reply.username.clone();

        let published_at = reply.published_at.clone();

        let published_at_dt = match Local.datetime_from_str(&published_at, "%d/%m/%Y %H:%M") {
            Ok(v) => v,
            Err(e) => now,
        };
        let time = published_at_format(&(now - published_at_dt));
        (replier_name, time)
    }

    fn build_separator_arguments(&mut self) -> (usize, usize, String) {
        let width = self.body_width();
        let rustbox = self.rustbox;

        let separator_width = if rustbox.width() >= 2 {
            rustbox.width() - 2
        } else {
            0
        };
        let separator_padding_width = if rustbox.width() > separator_width {
            rustbox.width() - separator_width
        } else {
            0
        } / 2;

        let separator_padding = seq_str_gen(0, separator_padding_width, " ", "");

        (separator_width, separator_padding_width, separator_padding)
    }

    fn build_separator_top(&mut self, replier_name: &str, time: &str) -> String {
        let replier_max_width = 14;
        let time_max_width = 5;
        let (separator_width, separator_padding_width, separator_padding) =
            self.build_separator_arguments();
        make_separator_top(separator_width,
                           &separator_padding,
                           replier_max_width,
                           &replier_name,
                           time_max_width,
                           &time)
    }

    fn build_separator_bottom(&mut self) -> String {
        let (separator_width, separator_padding_width, separator_padding) =
            self.build_separator_arguments();
        make_separator_bottom(separator_width, &separator_padding)
    }

    pub fn resetY(&mut self) {
        self.scrollY = 0;
    }

    pub fn scrollUp(&mut self, value: usize) -> bool {
        let tmp = self.scrollY;
        if tmp > value {
            self.scrollY = tmp - value;
            true
        } else if tmp != 0 {
            self.scrollY = 0;
            true
        } else {
            false
        }
    }

    pub fn scrollDown(&mut self, value: usize) -> bool {
        let tmp = self.scrollY;
        if tmp < 10000 {
            self.scrollY = tmp + value;
            return true;
        }
        false
    }

    pub fn body_height(&self) -> usize {
        if self.rustbox.height() >= 3 {
            self.rustbox.height() - 3
        } else {
            0
        }
    }

    pub fn body_width(&self) -> usize {
        if self.rustbox.width() >= 2 {
            self.rustbox.width() - 2
        } else {
            0
        }
    }
}

fn print_default(rustbox: &rustbox::RustBox, x: usize, y: usize, s: String) {
    rustbox.print(0, y, rustbox::RB_NORMAL, Color::White, Color::Black, &s);
}

fn make_separator_replier_name(separator_width: usize,
                               separator_padding: &str,
                               replier_max_width: usize,
                               replier_name: &str)
                               -> String {
    let replier_name_len = jks_len(&replier_name);
    let replier_name_spacing_width = replier_max_width - replier_name_len;
    let is_replier_name_spacing_width_odd = replier_name_spacing_width & 1 == 1;
    let replier_name_right_spacing_width = replier_name_spacing_width / 2;
    let replier_name_left_spacing_width = if is_replier_name_spacing_width_odd {
        replier_name_right_spacing_width + 1
    } else {
        replier_name_right_spacing_width
    };

    let replier_name_left_spacing = seq_str_gen(0, replier_name_left_spacing_width, "─", "");
    let replier_name_right_spacing = seq_str_gen(0, replier_name_right_spacing_width, "─", "");

    let separator_replier = format!("{}{}{}{}{}",
                                    "╭",
                                    replier_name_left_spacing,
                                    replier_name,
                                    replier_name_right_spacing,
                                    "");

    return separator_replier;
}

fn make_separator_time(separator_width: usize,
                       separator_padding: &str,
                       time_max_width: usize,
                       time: &str)
                       -> String {
    let time_len = jks_len(&time);
    let time_spacing_width = if time_max_width > time_len {
        time_max_width - time_len
    } else {
        0
    };

    let is_time_spacing_width_odd = time_spacing_width & 1 == 1;
    let time_right_spacing_width = time_spacing_width / 2;
    let time_left_spacing_width = if is_time_spacing_width_odd {
        time_right_spacing_width + 1
    } else {
        time_right_spacing_width
    };

    let time_left_spacing = seq_str_gen(0, time_left_spacing_width, "─", "");
    let time_right_spacing = seq_str_gen(0, time_right_spacing_width, "─", "");

    let separator_time = format!("{}{}{}{}{}",
                                 "",
                                 time_left_spacing,
                                 time,
                                 time_right_spacing,
                                 "╮");


    return separator_time;
}

fn make_separator_top(separator_width: usize,
                      separator_padding: &str,
                      replier_max_width: usize,
                      replier_name: &str,
                      time_max_width: usize,
                      time: &str)
                      -> String {

    let separator_replier = make_separator_replier_name(separator_width,
                                                        &separator_padding,
                                                        replier_max_width,
                                                        &replier_name);

    let separator_replier_width = jks_len(&separator_replier);

    let separator_time = make_separator_time(separator_width,
                                             &separator_padding,
                                             time_max_width,
                                             &time);

    let separator_time_width = jks_len(&separator_time);

    let separator_top_middle_width = if separator_width >=
                                        (separator_replier_width + separator_time_width) {
        separator_width - separator_replier_width - separator_time_width
    } else {
        0
    };

    let separator_top_middle = seq_str_gen(0, separator_top_middle_width, " ", "");
    let separator_top = format!("{}{}{}{}{}",
                                separator_padding,
                                separator_top_middle,
                                separator_replier,
                                separator_time,
                                separator_padding);
    return separator_top;
}

fn make_separator_bottom(separator_width: usize, separator_padding: &str) -> String {
    let style_box_width = 1;
    let separator_bottom_middle_width = if separator_width > style_box_width {
        separator_width - style_box_width
    } else {
        0
    };
    let separator_bottom_middle = seq_str_gen(0, separator_bottom_middle_width, "─", "");

    let separator_bottom = format!("{}{}{}{}",
                                   separator_padding,
                                   separator_bottom_middle,
                                   "╯",
                                   separator_padding);
    return separator_bottom;
}


fn published_at_format(duration: &Duration) -> String {
    let weeks = duration.num_weeks();
    let days = duration.num_days();
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();

    if weeks > 0 {
        format!("{}w", weeks)
    } else if days > 0 {
        format!("{}d", days)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        String::from("1m")
    }
}

fn seq_str_gen(start: usize, end: usize, sym: &str, join_sym: &str) -> String {
    (start..end).map(|_| sym.clone()).collect::<Vec<_>>().join(&join_sym)
}
