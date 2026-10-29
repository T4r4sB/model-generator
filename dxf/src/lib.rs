/// Gemini-generated

/// Комментарий от меня:
/// Я заебался тащить зависимости, которые тащат зависимости,
/// в которых какой-то очередной соевый долбоёб случайно нисхуя
/// потребовал самый новый компилятор,
/// причём даже указание конкретной версии не помогает, по итогу
/// приходится обновлять винду,
/// комп и так далее, и главное что всё равно вообще не факт что
///итоговый файл откроется в Автокаде и не завалит CypCut.
/// Слава богу, гугловский ИИ в бесплатной версии мне подсказал
/// как написать сохранялку без хуйни
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct DxfR12Writer {
    writer: BufWriter<File>,
    closed: bool,
}

impl DxfR12Writer {
    pub fn new(filename: &std::path::Path) -> std::io::Result<Self> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        // 1. СЕКЦИЯ HEADER: Указываем Систему Мер (миллиметры)
        // Код 9 — имя переменной, код 70 со значением 1 — Метрическая система (мм)
        writeln!(writer, "  0\nSECTION\n  2\nHEADER")?;
        writeln!(writer, "  9\n$MEASUREMENT\n 70\n1")?;
        writeln!(writer, "  0\nENDSEC")?;

        writeln!(writer, "  0\nSECTION\n  2\nTABLES")?;
        writeln!(writer, "  0\nTABLE\n  2\nLAYER\n 70\n1")?;
        writeln!(writer, "  0\nLAYER\n  2\n0\n 70\n0\n 62\n7\n370\n0")?;
        writeln!(writer, "  0\nENDTAB")?;
        writeln!(writer, "  0\nENDSEC")?;

/*
          0
SECTION
  2
TABLES
  0
TABLE
  2
LAYER
 70
1
  0
LAYER
  2
0
 70
0
 62
7
370
0
  0
ENDTAB
  0
ENDSEC*/

        // 2. ОТКРЫВАЕМ СЕКЦИЮ ENTITIES (Геометрия)
        writeln!(writer, "  0\nSECTION\n  2\nENTITIES")?;

        let closed = false;

        Ok(DxfR12Writer { writer, closed })
    }

    // Метод для записи простой линии (LINE)
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> std::io::Result<()> {
        writeln!(self.writer, "  0\nLINE\n  8\n0")?; // Слой 0
                                                     // writeln!(self.writer, " 62\n{}", 253)?;
        writeln!(self.writer, " 10\n{}\n 20\n{}", x1, y1)?; // Старт (X, Y)
        writeln!(self.writer, " 11\n{}\n 21\n{}", x2, y2)?; // Конец (X, Y)
        Ok(())
    }

    // Метод для записи окружности (CIRCLE)
    pub fn add_circle(&mut self, x: f64, y: f64, r: f64) -> std::io::Result<()> {
        writeln!(self.writer, "  0\nCIRCLE\n  8\n0")?; // Слой 0
                                                       // writeln!(self.writer, " 62\n{}", 253)?;
        writeln!(self.writer, " 10\n{}\n 20\n{}", x, y)?; // Центр (X, Y)
        writeln!(self.writer, " 40\n{}", r)?; // Радиус (R)
        Ok(())
    }

    // Метод для записи ПОЛИЛИНИИ (POLYLINE)
    // vertices: слайс пар координат &[(x, y)]
    // closed: true — замкнуть контур (соединить последнюю точку с первой), false — оставить разомкнутым
    pub fn add_polyline<I: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        vertices: I,
        closed: bool,
    ) -> std::io::Result<()> {
        // Объявляем объект POLYLINE
        writeln!(self.writer, "  0\nPOLYLINE\n  8\n0")?; // Слой 0
                                                         // writeln!(self.writer, " 62\n{}", 253)?;
        writeln!(self.writer, " 66\n1")?;

        // Код 70 отвечает за флаги полилинии. Значение 1 — замкнутая, 0 — разомкнутая.
        let flag = if closed { 1 } else { 0 };
        writeln!(self.writer, " 70\n{}", flag)?;

        // Записываем каждую точку в виде отдельного суб-объекта VERTEX
        for (x, y) in vertices {
            writeln!(self.writer, "  0\nVERTEX\n  8\n0")?;
            writeln!(self.writer, " 10\n{}\n 20\n{}", x, y)?;
        }

        // Обязательный маркер SEQEND, завершающий последовательность точек полилинии
        writeln!(self.writer, "  0\nSEQEND")?;
        Ok(())
    }

    // Метод закрытия файла (Закрываем ENTITIES и пишем маркер конца файла EOF)
    pub fn close(mut self) -> std::io::Result<()> {
        self.closed = true;
        writeln!(self.writer, "  0\nENDSEC\n  0\nEOF")?;
        Ok(())
    }
}

impl Drop for DxfR12Writer {
    fn drop(&mut self) {
        if !self.closed {
            panic!("DxfR12Writer was not closed!");
        }
    }
}
