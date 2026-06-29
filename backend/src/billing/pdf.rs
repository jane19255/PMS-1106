use super::models::{ClinicalSummary, MedicalReport};
use crate::models::PatientView;
use printpdf::{
    BuiltinFont, Color, IndirectFontRef, Line, Mm, PdfDocument, PdfLayerReference, Point, Rgb,
};
use std::io::{BufWriter, Cursor};

const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const LEFT: f32 = 20.0;
const RIGHT: f32 = 20.0;

pub fn render_medical_report_pdf(
    report: &MedicalReport,
    patient: Option<&PatientView>,
    clinical: Option<&ClinicalSummary>,
) -> Result<Vec<u8>, String> {
    // Build each part in order because they all share the same vertical position.
    let (document, page, layer) =
        PdfDocument::new("Invoice", Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Page 1");
    let font = document
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|error| error.to_string())?;
    let bold = document
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|error| error.to_string())?;
    let layer = document.get_page(page).get_layer(layer);

    let mut pdf = InvoicePdf::new(layer);
    pdf.clinic_header(&font, &bold);
    pdf.invoice_title(report, &font, &bold);
    pdf.bill_to(patient, report.invoice.patient_id.as_str(), &font, &bold);
    pdf.invoice_table(report, &font, &bold);
    pdf.payment_summary(report, &font, &bold);
    pdf.medical_reference(clinical, &font, &bold);
    pdf.footer(&font);

    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    document
        .save(&mut buffer)
        .map_err(|error| error.to_string())?;
    buffer
        .into_inner()
        .map_err(|error| error.to_string())
        .map(Cursor::into_inner)
}

struct InvoicePdf {
    layer: PdfLayerReference,
    // Current writing position, measured from the bottom of the page.
    y: f32,
}

impl InvoicePdf {
    fn new(layer: PdfLayerReference) -> Self {
        Self {
            layer,
            y: PAGE_HEIGHT - 20.0,
        }
    }

    fn clinic_header(&mut self, font: &IndirectFontRef, bold: &IndirectFontRef) {
        self.text_right("CSCare Clinic", PAGE_WIDTH - RIGHT, self.y, 11.0, bold);
        self.text_right(
            "1 Punggol Coast Road",
            PAGE_WIDTH - RIGHT,
            self.y - 7.0,
            8.0,
            font,
        );
        self.text_right(
            "Singapore 828608",
            PAGE_WIDTH - RIGHT,
            self.y - 13.0,
            8.0,
            font,
        );
        self.text_right(
            "Phone: +65 9123 5678 | Email: info@cscare.com",
            PAGE_WIDTH - RIGHT,
            self.y - 19.0,
            8.0,
            font,
        );
        self.text_right(
            "Website: www.cscare.com",
            PAGE_WIDTH - RIGHT,
            self.y - 25.0,
            8.0,
            font,
        );
        self.y -= 42.0;
    }

    fn invoice_title(
        &mut self,
        report: &MedicalReport,
        font: &IndirectFontRef,
        bold: &IndirectFontRef,
    ) {
        self.text("INVOICE", 22.0, bold);
        self.text(
            "Medical services, treatment, and prescription billing",
            8.5,
            font,
        );

        let info_x = PAGE_WIDTH - RIGHT - 65.0;
        let top = self.y + 19.0;
        self.text_at("Invoice No.", info_x, top, 8.0, bold);
        self.text_at(report.invoice.id.as_str(), info_x + 28.0, top, 8.0, font);
        self.text_at("Invoice Date", info_x, top - 7.0, 8.0, bold);
        self.text_at(
            &report.invoice.created_at.date_naive().to_string(),
            info_x + 28.0,
            top - 7.0,
            8.0,
            font,
        );
        self.text_at("Status", info_x, top - 14.0, 8.0, bold);
        self.text_at(
            &format!("{:?}", report.invoice.status),
            info_x + 28.0,
            top - 14.0,
            8.0,
            font,
        );
        self.line(LEFT, self.y - 4.0, PAGE_WIDTH - RIGHT, self.y - 4.0);
        self.y -= 18.0;
    }

    fn bill_to(
        &mut self,
        patient: Option<&PatientView>,
        patient_id: &str,
        font: &IndirectFontRef,
        bold: &IndirectFontRef,
    ) {
        self.text("Bill To", 10.0, bold);

        if let Some(patient) = patient {
            self.text(
                &format!("{} {}", patient.first_name, patient.last_name),
                9.0,
                bold,
            );
            self.text(&format!("Patient ID: {}", patient.id), 8.0, font);
            self.text(&format!("IC Number: {}", patient.nric), 8.0, font);
            self.text(&format!("Phone: {}", patient.phone), 8.0, font);

            let right_x = LEFT + 92.0;
            let base_y = self.y + 27.0;
            self.text_at("Patient Details", right_x, base_y, 10.0, bold);
            self.text_at(
                &format!("DOB: {}", patient.dob),
                right_x,
                base_y - 7.0,
                8.0,
                font,
            );
            self.text_at(
                &format!(
                    "Allergies: {}",
                    patient.allergies.as_deref().unwrap_or("None recorded")
                ),
                right_x,
                base_y - 14.0,
                8.0,
                font,
            );
            self.text_at(
                &format!(
                    "Conditions: {}",
                    patient.conditions.as_deref().unwrap_or("None recorded")
                ),
                right_x,
                base_y - 21.0,
                8.0,
                font,
            );
        } else {
            self.text(&format!("Patient ID: {patient_id}"), 8.5, font);
        }

        self.y -= 10.0;
    }

    fn invoice_table(
        &mut self,
        report: &MedicalReport,
        font: &IndirectFontRef,
        bold: &IndirectFontRef,
    ) {
        self.section_heading("Itemized Charges", bold);
        self.table_header(
            &["Category", "Description", "Amount"],
            &[40.0, 96.0, 34.0],
            bold,
        );

        for item in &report.invoice.items {
            self.table_row(
                &[
                    &format!("{:?}", item.item_type),
                    item.name.as_str(),
                    &format!("${:.2}", item.cost),
                ],
                &[40.0, 96.0, 34.0],
                font,
            );
        }
    }

    fn payment_summary(
        &mut self,
        report: &MedicalReport,
        font: &IndirectFontRef,
        bold: &IndirectFontRef,
    ) {
        self.y -= 8.0;
        let x = PAGE_WIDTH - RIGHT - 70.0;
        self.summary_line("Subtotal", report.invoice.subtotal, x, font, bold);
        self.summary_line("Total Charges", report.invoice.total, x, font, bold);
        self.summary_line("Amount Paid", report.invoice.amount_paid, x, font, bold);
        self.line(x, self.y, PAGE_WIDTH - RIGHT, self.y);
        self.y -= 7.0;
        self.text_at("Balance Due", x, self.y, 10.0, bold);
        self.text_right(
            &format!("${:.2}", report.invoice.balance_due),
            PAGE_WIDTH - RIGHT,
            self.y,
            10.0,
            bold,
        );
        self.y -= 12.0;

        self.section_heading("Payment Record", bold);
        if report.invoice.payments.is_empty() {
            self.text("No payments recorded.", 8.5, font);
        } else {
            self.table_header(
                &["Date", "Method", "Reference", "Amount"],
                &[56.0, 36.0, 46.0, 32.0],
                bold,
            );
            for payment in &report.invoice.payments {
                self.table_row(
                    &[
                        &payment.paid_at.date_naive().to_string(),
                        payment.payment_method.as_str(),
                        payment
                            .transaction_reference
                            .as_deref()
                            .unwrap_or("Not recorded"),
                        &format!("${:.2}", payment.amount),
                    ],
                    &[56.0, 36.0, 46.0, 32.0],
                    font,
                );
            }
        }
    }

    fn medical_reference(
        &mut self,
        clinical: Option<&ClinicalSummary>,
        font: &IndirectFontRef,
        bold: &IndirectFontRef,
    ) {
        self.y -= 8.0;
        self.section_heading("Medical Reference", bold);
        self.text(
            &format!(
                "Doctor: {}",
                clinical
                    .and_then(|record| record.doctor_name.as_deref())
                    .unwrap_or("Not recorded")
            ),
            8.0,
            font,
        );
        self.text(
            &format!(
                "Diagnosis: {}",
                clinical
                    .and_then(|record| record.diagnosis.as_deref())
                    .unwrap_or("Not recorded")
            ),
            8.0,
            font,
        );
    }

    fn footer(&self, font: &IndirectFontRef) {
        self.line(LEFT, 24.0, PAGE_WIDTH - RIGHT, 24.0);
        self.text_center(
            "Thank you for choosing CSCare Clinic. Please settle outstanding balances by the due date shown by reception.",
            16.0,
            7.5,
            font,
        );
    }

    fn section_heading(&mut self, title: &str, bold: &IndirectFontRef) {
        self.text(title, 11.0, bold);
        self.line(LEFT, self.y + 2.0, PAGE_WIDTH - RIGHT, self.y + 2.0);
        self.y -= 5.0;
    }

    fn table_header(&mut self, values: &[&str], widths: &[f32], font: &IndirectFontRef) {
        self.table_row(values, widths, font);
        self.line(LEFT, self.y + 2.0, PAGE_WIDTH - RIGHT, self.y + 2.0);
    }

    fn table_row(&mut self, values: &[&str], widths: &[f32], font: &IndirectFontRef) {
        // Move across the row using each column width instead of fixed positions.
        let mut x = LEFT;
        for (index, (value, width)) in values.iter().zip(widths.iter()).enumerate() {
            if index == values.len() - 1 {
                self.text_right(value, PAGE_WIDTH - RIGHT, self.y, 8.2, font);
            } else {
                self.text_at(&truncate(value, 42), x, self.y, 8.2, font);
            }
            x += width;
        }
        self.y -= 7.0;
    }

    fn summary_line(
        &mut self,
        label: &str,
        amount: f64,
        x: f32,
        font: &IndirectFontRef,
        bold: &IndirectFontRef,
    ) {
        self.text_at(label, x, self.y, 8.5, font);
        self.text_right(
            &format!("${:.2}", amount),
            PAGE_WIDTH - RIGHT,
            self.y,
            8.5,
            bold,
        );
        self.y -= 7.0;
    }

    fn text(&mut self, text: &str, size: f32, font: &IndirectFontRef) {
        self.text_at(text, LEFT, self.y, size, font);
        self.y -= size * 0.52 + 2.0;
    }

    fn text_at(&self, text: &str, x: f32, y: f32, size: f32, font: &IndirectFontRef) {
        self.layer
            .set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        self.layer.use_text(text, size, Mm(x), Mm(y), font);
    }

    fn text_right(&self, text: &str, x: f32, y: f32, size: f32, font: &IndirectFontRef) {
        // printpdf does not align text for us, so estimate its width first.
        self.text_at(text, x - estimate_text_width(text, size), y, size, font);
    }

    fn text_center(&self, text: &str, y: f32, size: f32, font: &IndirectFontRef) {
        self.text_at(
            text,
            (PAGE_WIDTH - estimate_text_width(text, size)) / 2.0,
            y,
            size,
            font,
        );
    }

    fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.layer
            .set_outline_color(Color::Rgb(Rgb::new(0.82, 0.82, 0.82, None)));
        self.layer.add_line(Line {
            points: vec![
                (Point::new(Mm(x1), Mm(y1)), false),
                (Point::new(Mm(x2), Mm(y2)), false),
            ],
            is_closed: false,
        });
    }
}

fn truncate(text: &str, limit: usize) -> String {
    // Keep long database values from overflowing into the next PDF column.
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let mut value: String = text.chars().take(limit.saturating_sub(3)).collect();
    value.push_str("...");
    value
}

fn estimate_text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.36
}
