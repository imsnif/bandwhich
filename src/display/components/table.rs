use std::{collections::HashMap, net::IpAddr, ops::Index, rc::Rc};

use derive_more::Debug;
use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    display::{Bandwidth, BandwidthUnitFamily, DisplayBandwidth, UIState},
    network::{display_connection_string, display_ip_or_host},
};

/// The displayed layout choice of a table.
/// Each value in the array is the width of each column.
///
/// Note that this only determines how a table is displayed, not what data it contains.
///
/// If we intend to display different number of columns in the future,
/// then new variants should be added.
#[derive(Copy, Clone, Debug)]
pub enum DisplayLayout {
    /// Show 2 columns.
    C2([u16; 2]),
    /// Show 3 columns.
    C3([u16; 3]),
    /// Show 4 columns.
    C4([u16; 4]),
    /// Show 5 columns.
    C5([u16; 5]),
    /// Show 6 columns.
    C6([u16; 6]),
}

impl Index<usize> for DisplayLayout {
    type Output = u16;

    fn index(&self, i: usize) -> &Self::Output {
        match self {
            Self::C2(arr) => &arr[i],
            Self::C3(arr) => &arr[i],
            Self::C4(arr) => &arr[i],
            Self::C5(arr) => &arr[i],
            Self::C6(arr) => &arr[i],
        }
    }
}

impl DisplayLayout {
    #[inline]
    fn columns_count(&self) -> usize {
        match self {
            Self::C2(_) => 2,
            Self::C3(_) => 3,
            Self::C4(_) => 4,
            Self::C5(_) => 5,
            Self::C6(_) => 6,
        }
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &u16> {
        match self {
            Self::C2(ws) => ws.iter(),
            Self::C3(ws) => ws.iter(),
            Self::C4(ws) => ws.iter(),
            Self::C5(ws) => ws.iter(),
            Self::C6(ws) => ws.iter(),
        }
    }

    #[inline]
    fn widths_sum(&self) -> u16 {
        self.iter().sum()
    }

    /// Returns the computed actual width and the spacer width.
    ///
    /// See [`Table`] for layout rules.
    fn compute_actual_widths(&self, available: u16) -> (Self, u16) {
        let columns_count = self.columns_count() as u16;
        let desired_min = self.widths_sum();

        // spacer max width is 2
        let spacer = if available > desired_min {
            ((available - desired_min) / (columns_count - 1)).min(2)
        } else {
            0
        };
        let available_without_spacers = available - spacer * (columns_count - 1);

        // multiplier
        let m = available_without_spacers as f64 / desired_min as f64;

        // remainder width is arbitrarily given to column 0
        let computed = match *self {
            Self::C2([_w0, w1]) => {
                let w1_new = (w1 as f64 * m).trunc() as u16;
                Self::C2([available_without_spacers - w1_new, w1_new])
            }
            Self::C3([_w0, w1, w2]) => {
                let w1_new = (w1 as f64 * m).trunc() as u16;
                let w2_new = (w2 as f64 * m).trunc() as u16;
                Self::C3([available_without_spacers - w1_new - w2_new, w1_new, w2_new])
            }
            Self::C4([_w0, w1, w2, w3]) => {
                let w1_new = (w1 as f64 * m).trunc() as u16;
                let w2_new = (w2 as f64 * m).trunc() as u16;
                let w3_new = (w3 as f64 * m).trunc() as u16;
                Self::C4([
                    available_without_spacers - w1_new - w2_new - w3_new,
                    w1_new,
                    w2_new,
                    w3_new,
                ])
            }
            Self::C5([_w0, w1, w2, w3, w4]) => {
                let w1_new = (w1 as f64 * m).trunc() as u16;
                let w2_new = (w2 as f64 * m).trunc() as u16;
                let w3_new = (w3 as f64 * m).trunc() as u16;
                let w4_new = (w4 as f64 * m).trunc() as u16;
                Self::C5([
                    available_without_spacers - w1_new - w2_new - w3_new - w4_new,
                    w1_new,
                    w2_new,
                    w3_new,
                    w4_new,
                ])
            }
            Self::C6([_w0, w1, w2, w3, w4, w5]) => {
                let w1_new = (w1 as f64 * m).trunc() as u16;
                let w2_new = (w2 as f64 * m).trunc() as u16;
                let w3_new = (w3 as f64 * m).trunc() as u16;
                let w4_new = (w4 as f64 * m).trunc() as u16;
                let w5_new = (w5 as f64 * m).trunc() as u16;
                Self::C6([
                    available_without_spacers - w1_new - w2_new - w3_new - w4_new - w5_new,
                    w1_new,
                    w2_new,
                    w3_new,
                    w4_new,
                    w5_new,
                ])
            }
        };

        (computed, spacer)
    }
}

/// All data of a table.
///
/// If tables with different number of columns are added in the future,
/// then new variants should be added.
#[derive(Clone, Debug)]
enum TableData {
    /// A table with 3 columns.
    C3(NColsTableData<3>),
    /// A table with 4 columns.
    C4(NColsTableData<4>),
    /// A table with 5 columns.
    C5(NColsTableData<5>),
    /// A table with 6 columns.
    C6(NColsTableData<6>),
}

impl From<NColsTableData<3>> for TableData {
    fn from(data: NColsTableData<3>) -> Self {
        Self::C3(data)
    }
}

impl From<NColsTableData<4>> for TableData {
    fn from(data: NColsTableData<4>) -> Self {
        Self::C4(data)
    }
}

impl From<NColsTableData<5>> for TableData {
    fn from(data: NColsTableData<5>) -> Self {
        Self::C5(data)
    }
}

impl From<NColsTableData<6>> for TableData {
    fn from(data: NColsTableData<6>) -> Self {
        Self::C6(data)
    }
}

impl TableData {
    fn column_names(&self) -> &[&str] {
        match self {
            Self::C3(inner) => &inner.column_names,
            Self::C4(inner) => &inner.column_names,
            Self::C5(inner) => &inner.column_names,
            Self::C6(inner) => &inner.column_names,
        }
    }

    fn rows(&self) -> Vec<&[String]> {
        match self {
            Self::C3(inner) => inner.rows.iter().map(|r| r.as_slice()).collect(),
            Self::C4(inner) => inner.rows.iter().map(|r| r.as_slice()).collect(),
            Self::C5(inner) => inner.rows.iter().map(|r| r.as_slice()).collect(),
            Self::C6(inner) => inner.rows.iter().map(|r| r.as_slice()).collect(),
        }
    }

    fn column_selector(&self) -> &dyn Fn(&DisplayLayout) -> Vec<usize> {
        match self {
            Self::C3(inner) => inner.column_selector.as_ref(),
            Self::C4(inner) => inner.column_selector.as_ref(),
            Self::C5(inner) => inner.column_selector.as_ref(),
            Self::C6(inner) => inner.column_selector.as_ref(),
        }
    }
}

/// All data of a table with `C` columns.
///
/// Note that the number of columns here is independent of the number of columns
/// being actually shown. If width-constrained, we might only show some of the columns.
#[derive(Clone, Debug)]
struct NColsTableData<const C: usize> {
    /// The name of each column.
    column_names: [&'static str; C],
    /// All rows of data.
    rows: Vec<[String; C]>,
    /// Function to determine which columns to show for a given layout.
    ///
    /// This function should return a vector of column indices.
    /// The indices should be less than `C`; otherwise this will cause a runtime panic.
    #[debug("Rc</* function pointer */>")]
    column_selector: Rc<ColumnSelectorFn>,
}

/// Clippy wanted me to write this. 💢
type ColumnSelectorFn = dyn Fn(&DisplayLayout) -> Vec<usize>;

/// A table displayed by bandwhich.
#[derive(Clone, Debug)]
pub struct Table {
    title: &'static str,
    /// A layout mapping between minimum available width and the width of each column.
    ///
    /// Note that the width of each column here is the "desired minimum width".
    ///
    /// - Wt = available width of table
    /// - Wd = sum of desired minimum width of each column
    ///
    /// - If `Wt >= Wd`, spacers with a maximum width of `2` will be inserted
    ///   between columns; and then the columns will proportionally expand.
    /// - If `Wt < Wd`, columns will proportionally shrink.
    width_cutoffs: Vec<(u16, DisplayLayout)>,
    data: TableData,
    process_chart_rows: Option<Vec<ProcessChartRow>>,
    chart_column_indices: Option<Vec<usize>>,
}

#[derive(Clone, Debug)]
struct ProcessChartRow {
    upload: Vec<(f64, f64)>,
    download: Vec<(f64, f64)>,
    max_x: f64,
}

impl Table {
    pub fn create_connections_table(state: &UIState, ip_to_host: &HashMap<IpAddr, String>) -> Self {
        use DisplayLayout as D;

        let title = "Utilization by connection";
        let width_cutoffs = vec![
            (0, D::C2([32, 18])),
            (80, D::C3([36, 12, 18])),
            (100, D::C3([54, 18, 22])),
            (120, D::C3([72, 24, 22])),
        ];

        let column_names = [
            "Connection",
            "Process",
            if state.cumulative_mode {
                "Data (Up / Down)"
            } else {
                "Rate (Up / Down)"
            },
        ];
        let rows = state
            .connections
            .iter()
            .map(|(connection, connection_data)| {
                [
                    display_connection_string(
                        connection,
                        ip_to_host,
                        &connection_data.interface_name,
                    ),
                    connection_data.process_name.to_string(),
                    display_upload_and_download(
                        connection_data,
                        state.unit_family,
                        state.cumulative_mode,
                    ),
                ]
            })
            .collect();
        let column_selector = Rc::new(|layout: &D| match layout {
            D::C2(_) => vec![0, 2],
            D::C3(_) => vec![0, 1, 2],
            D::C4(_) => unreachable!(),
            D::C5(_) => unreachable!(),
            D::C6(_) => unreachable!(),
        });

        Table {
            title,
            width_cutoffs,
            data: NColsTableData {
                column_names,
                rows,
                column_selector,
            }
            .into(),
            process_chart_rows: None,
            chart_column_indices: None,
        }
    }

    pub fn create_processes_table(state: &UIState) -> Self {
        use DisplayLayout as D;

        let title = "Utilization by process name";
        let width_cutoffs = vec![
            (0, D::C2([16, 18])),
            (50, D::C3([16, 12, 20])),
            (60, D::C3([24, 12, 20])),
            (80, D::C4([28, 12, 12, 24])),
            (120, D::C6([28, 8, 12, 18, 18, 18])),
        ];

        let column_names = [
            "Process",
            "PID",
            "Connections",
            if state.cumulative_mode {
                "Data (Up / Down)"
            } else {
                "Rate (Up / Down)"
            },
            "Upload",
            "Download",
        ];
        let rows = state
            .processes
            .iter()
            .map(|(proc_info, data_for_process)| {
                [
                    proc_info.name.to_string(),
                    proc_info.pid.to_string(),
                    data_for_process.connection_count.to_string(),
                    display_upload_and_download(
                        data_for_process,
                        state.unit_family,
                        state.cumulative_mode,
                    ),
                    String::new(),
                    String::new(),
                ]
            })
            .collect();
        let process_chart_rows = Some(
            state
                .processes
                .iter()
                .map(|(proc_info, _)| build_process_chart_row(state, proc_info))
                .collect(),
        );
        let column_selector = Rc::new(|layout: &D| match layout {
            D::C2(_) => vec![0, 3],
            D::C3(_) => vec![0, 2, 3],
            D::C4(_) => vec![0, 1, 2, 3],
            D::C5(_) => unreachable!(),
            D::C6(_) => vec![0, 1, 2, 3, 4, 5],
        });

        Table {
            title,
            width_cutoffs,
            data: NColsTableData {
                column_names,
                rows,
                column_selector,
            }
            .into(),
            process_chart_rows,
            chart_column_indices: Some(vec![4, 5]),
        }
    }

    pub fn create_remote_addresses_table(
        state: &UIState,
        ip_to_host: &HashMap<IpAddr, String>,
    ) -> Self {
        use DisplayLayout as D;

        let title = "Utilization by remote address";
        let width_cutoffs = vec![
            (0, D::C2([16, 16])),
            (40, D::C2([20, 16])),
            (60, D::C3([24, 10, 20])),
            (100, D::C3([54, 16, 24])),
        ];

        let column_names = [
            "Remote Address",
            "Connections",
            if state.cumulative_mode {
                "Data (Up / Down)"
            } else {
                "Rate (Up / Down)"
            },
        ];
        let rows = state
            .remote_addresses
            .iter()
            .map(|(remote_address, data_for_remote_address)| {
                let remote_address = display_ip_or_host(*remote_address, ip_to_host);
                [
                    remote_address,
                    data_for_remote_address.connection_count.to_string(),
                    display_upload_and_download(
                        data_for_remote_address,
                        state.unit_family,
                        state.cumulative_mode,
                    ),
                ]
            })
            .collect();
        let column_selector = Rc::new(|layout: &D| match layout {
            D::C2(_) => vec![0, 2],
            D::C3(_) => vec![0, 1, 2],
            D::C4(_) => unreachable!(),
            D::C5(_) => unreachable!(),
            D::C6(_) => unreachable!(),
        });

        Table {
            title,
            width_cutoffs,
            data: NColsTableData {
                column_names,
                rows,
                column_selector,
            }
            .into(),
            process_chart_rows: None,
            chart_column_indices: None,
        }
    }

    /// See [`Table`] for layout rules.
    pub fn render(&self, frame: &mut Frame, rect: Rect) {
        let block = Block::default().title(self.title).borders(Borders::ALL);
        let inner = block.inner(rect);
        let (computed_layout, spacer_width) = {
            // pick the largest possible layout, constrained by the available width
            let &(_, layout) = self
                .width_cutoffs
                .iter()
                .rev()
                .find(|(cutoff, _)| rect.width > *cutoff)
                .unwrap(); // all cutoff tables have a 0-width entry
            layout.compute_actual_widths(inner.width)
        };

        let columns_to_show = self.data.column_selector()(&computed_layout);
        let column_names: Vec<_> = columns_to_show
            .iter()
            .copied()
            .map(|i| self.data.column_names()[i])
            .collect();
        let show_charts = self
            .chart_column_indices
            .as_ref()
            .map(|indices| indices.iter().any(|index| columns_to_show.contains(index)))
            .unwrap_or(false)
            && self.process_chart_rows.is_some();

        // text needs to react to column widths
        let tui_rows_iter = self
            .data
            .rows()
            .into_iter()
            .map(|row_data| {
                let shown_columns_data = columns_to_show.iter().copied().map(|i| &row_data[i]);
                let column_widths = computed_layout.iter().copied();
                shown_columns_data
                    .zip_eq(column_widths)
                    .map(|(text, width)| truncate_middle(text, width))
                    .collect::<Vec<_>>()
            })
            .map(Row::new);

        let widths_constraints: Vec<_> = computed_layout
            .iter()
            .copied()
            .map(Constraint::Length)
            .collect();

        let table = ratatui::widgets::Table::new(tui_rows_iter, widths_constraints)
            .block(block)
            .header(Row::new(column_names).style(Style::default().fg(Color::Yellow)))
            .flex(ratatui::layout::Flex::Legacy)
            .column_spacing(spacer_width);
        frame.render_widget(table, rect);

        if show_charts {
            self.render_process_charts(
                frame,
                inner,
                &computed_layout,
                spacer_width,
                &columns_to_show,
            );
        }
    }

    fn render_process_charts(
        &self,
        frame: &mut Frame,
        inner: Rect,
        computed_layout: &DisplayLayout,
        spacer_width: u16,
        columns_to_show: &[usize],
    ) {
        let Some(chart_rows) = &self.process_chart_rows else {
            return;
        };
        let Some(chart_column_indices) = self.chart_column_indices.as_ref() else {
            return;
        };
        let available_rows = inner.height.saturating_sub(1) as usize;
        let rows_to_render = chart_rows.len().min(available_rows);

        for (row_index, row_data) in chart_rows.iter().take(rows_to_render).enumerate() {
            let chart_y = inner.y + 1 + row_index as u16;
            let max_x = row_data.max_x.max(1.0);
            for &chart_column_index in chart_column_indices {
                let Some(display_index) = columns_to_show
                    .iter()
                    .position(|&i| i == chart_column_index)
                else {
                    continue;
                };
                let mut chart_x = inner.x;
                for col in 0..display_index {
                    chart_x = chart_x.saturating_add(computed_layout[col]);
                    chart_x = chart_x.saturating_add(spacer_width);
                }
                let chart_rect = Rect {
                    x: chart_x,
                    y: chart_y,
                    width: computed_layout[display_index],
                    height: 1,
                };
                let (datasets, max_y) = if chart_column_index == 4 {
                    (
                        vec![Dataset::default()
                            .name("up")
                            .marker(Marker::Braille)
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(Color::Blue))
                            .data(&row_data.upload)],
                        max_series_y(&row_data.upload).max(1.0),
                    )
                } else {
                    (
                        vec![Dataset::default()
                            .name("down")
                            .marker(Marker::Braille)
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(Color::Green))
                            .data(&row_data.download)],
                        max_series_y(&row_data.download).max(1.0),
                    )
                };
                let chart = Chart::new(datasets)
                    .x_axis(Axis::default().bounds([0.0, max_x]))
                    .y_axis(Axis::default().bounds([0.0, max_y]));
                frame.render_widget(chart, chart_rect);
            }
        }
    }
}

fn display_upload_and_download(
    bandwidth: &impl Bandwidth,
    unit_family: BandwidthUnitFamily,
    _cumulative: bool,
) -> String {
    let up = DisplayBandwidth {
        bandwidth: bandwidth.get_total_bytes_uploaded() as f64,
        unit_family,
    };
    let down = DisplayBandwidth {
        bandwidth: bandwidth.get_total_bytes_downloaded() as f64,
        unit_family,
    };
    format!("{up} / {down}")
}

fn build_process_chart_row(state: &UIState, proc_info: &crate::os::ProcessInfo) -> ProcessChartRow {
    let history = state.process_bandwidth_history(proc_info);
    if history.is_empty() {
        return ProcessChartRow {
            upload: vec![(0.0, 0.0)],
            download: vec![(0.0, 0.0)],
            max_x: 1.0,
        };
    }

    let mut upload = Vec::with_capacity(history.len());
    let mut download = Vec::with_capacity(history.len());
    for (idx, (up, down)) in history.into_iter().enumerate() {
        let x = idx as f64;
        let up_f = up as f64;
        let down_f = down as f64;
        upload.push((x, up_f));
        download.push((x, down_f));
    }

    let max_x = (upload.len().saturating_sub(1)) as f64;
    ProcessChartRow {
        upload,
        download,
        max_x,
    }
}

fn max_series_y(points: &[(f64, f64)]) -> f64 {
    points
        .iter()
        .fold(0.0, |max, (_, y)| if *y > max { *y } else { max })
}

fn collect_to_unicode_width<T>(iter: impl Iterator<Item = char>, width: usize) -> T
where
    T: FromIterator<char>,
{
    let mut chunk_width = 0;
    iter.take_while(|ch| {
        chunk_width += ch.width().unwrap_or(0);
        chunk_width <= width
    })
    .collect()
}

fn truncate_middle(row: &str, max_len: u16) -> String {
    const ELLIPSIS: &str = "..";

    if max_len < 6 {
        collect_to_unicode_width(row.chars(), max_len as usize)
    } else if row.width() as u16 > max_len {
        let suffix_len = (max_len as usize - ELLIPSIS.len()) / 2;
        // remainder length arbitrarily given to prefix
        let prefix_len = max_len as usize - ELLIPSIS.len() - suffix_len;

        let prefix: String = collect_to_unicode_width(row.chars(), prefix_len);
        let suffix: String = collect_to_unicode_width::<Vec<_>>(row.chars().rev(), suffix_len)
            .into_iter()
            .rev()
            .collect();
        format!("{prefix}{ELLIPSIS}{suffix}")
    } else {
        row.to_string()
    }
}
