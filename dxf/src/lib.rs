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
        writeln!(
            writer,
            "  0
SECTION
  2
HEADER
  9
$ACADVER
  1
AC1009
  9
$MEASUREMENT
 70
1
  9
$LWDISPLAY
 290
0        <-- ВЫКЛЮЧАЕТ ОТОБРАЖЕНИЕ ТОЛЩИНЫ ДЛЯ ВСЕХ САПР
  0
ENDSEC
  0
SECTION
  2
ENTITIES"
        )?;

        let closed = false;

        Ok(DxfR12Writer { writer, closed })
    }

    // Метод для записи простой линии (LINE)
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> std::io::Result<()> {
        writeln!(
            self.writer,
            "  0
LINE
  8
0
 10
{x1}
 20
{y1}
 11
{x2}
 21
{y2}"
        )?;
        Ok(())
    }

    // Метод для записи окружности (CIRCLE)
    pub fn add_circle(&mut self, x: f64, y: f64, r: f64) -> std::io::Result<()> {
        writeln!(
            self.writer,
            "  0
CIRCLE
  8
0
 10
{x}
 20
{y}
 40
{r}"
        )?;
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

        let flag = if closed { 1 } else { 0 };
        writeln!(
            self.writer,
            "  0
POLYLINE
  8
0
 66
1
 70
{flag}"
        )?;

        // Записываем каждую точку в виде отдельного суб-объекта VERTEX
        for (x, y) in vertices {
            writeln!(
                self.writer,
                "  0
VERTEX
  8
0
 10
{x}
 20
{y}"
            )?;
        }

        // Обязательный маркер SEQEND, завершающий последовательность точек полилинии
        writeln!(
            self.writer,
            "  0
SEQEND"
        )?;
        Ok(())
    }

    // Метод закрытия файла (Закрываем ENTITIES и пишем маркер конца файла EOF)
    pub fn close(mut self) -> std::io::Result<()> {
        self.closed = true;
        writeln!(
            self.writer,
            "  0
ENDSEC
  0
EOF"
        )?;
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
