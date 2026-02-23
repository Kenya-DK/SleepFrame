using Newtonsoft.Json.Linq;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SleepFrame
{
    public partial class RecordDialog : Form
    {
        public const int WM_NCLBUTTONDOWN = 0xA1;
        public const int HT_CAPTION = 0x2;

        [DllImportAttribute("User32.dll")]
        public static extern int SendMessage(IntPtr hWnd, int Msg, int wParam, int lParam);
        [DllImportAttribute("User32.dll")]
        public static extern bool ReleaseCapture();


        //Test
        [DllImport("User32.dll")]
        static extern IntPtr GetDC(IntPtr hwnd);

        [DllImport("User32.dll")]
        static extern void ReleaseDC(IntPtr dc);


        bool _IsLeftClik;
        List<Point> _CursorPoint = new List<Point>();
        List<Form> _FormList = new List<Form>();


        // We need to use unmanaged code
        [DllImport("user32.dll")]
        // GetCursorPos() makes everything possible
        static extern bool GetCursorPos(ref Point lpPoint);

        public RecordDialog(List<Point> _Points)
        {
            InitializeComponent();

            timer1.Start();
            if (_Points == null)
                return;
            _CursorPoint = _Points;
            txtRecotdsCount.Text = _CursorPoint.Count.ToString();           
        }

        public List<Point> GetCursorPointList()
        {
            return _CursorPoint;
        }

        private void timer1_Tick(object sender, EventArgs e)
        {
            // New point that will be updated by the function with the current coordinates

            Point defPnt = new Point();

            // Call the function and pass the Point, defPnt

            GetCursorPos(ref defPnt);

            // Now after calling the function, defPnt contains the coordinates which we can read

            txtY.Text = defPnt.X.ToString();

            txtX.Text = defPnt.Y.ToString();
        }

        private void lblRecord_MouseCaptureChanged(object sender, EventArgs e)
        {
            if (_IsLeftClik)
            {
                _CursorPoint.Add(Cursor.Position);
            }
            txtRecotdsCount.Text = _CursorPoint.Count.ToString();
        }

        private void lblRecord_MouseDown(object sender, MouseEventArgs e)
        {
            if (e.Button == System.Windows.Forms.MouseButtons.Left)
            {
                _IsLeftClik = true;
                Cursor.Current = Cursors.Cross;
            }
            else
            {
                _IsLeftClik = false;
                int i = 0;
                foreach (Point item in _CursorPoint)
                {
                    //ArrowDlg ad = new ArrowDlg();
                    //ad.Location = new Point(item.X, item.Y);
                    //ad.Size = new System.Drawing.Size(17,17);
                    //ad.Show();
                    IntPtr desktop = GetDC(IntPtr.Zero);
                    i++;
                    using (Graphics g = Graphics.FromHdc(desktop))
                    {
                        Pen pen = new Pen(Color.Lime, 8);
                        pen.StartCap = LineCap.ArrowAnchor;
                        pen.EndCap = LineCap.RoundAnchor;
                        g.DrawLine(pen, item.X, item.Y, item.X + 20, item.Y - 50);

                        Font drawFont = new Font("Arial", 16, FontStyle.Bold | FontStyle.Underline | FontStyle.Italic);
                        SolidBrush drawBrush = new SolidBrush(Color.LimeGreen);
                        StringFormat drawFormat = new StringFormat();
                        g.DrawString(i.ToString(), drawFont, drawBrush, item.X + 10, item.Y - 85, drawFormat);

                        drawFont.Dispose();
                        drawBrush.Dispose();
                        g.Dispose();
                    }

                }
                Cursor.Current = Cursors.Default;
            }

        }

        private void RecordDialog_MouseDown(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Left)
            {
                ReleaseCapture();
                SendMessage(Handle, WM_NCLBUTTONDOWN, HT_CAPTION, 0);
            }
        }

        private void BtnCopyList_Click(object sender, EventArgs e)
        {
            JArray array = new JArray();
            foreach (var item in _CursorPoint)
            {
                array.Add(item.X.ToString() +", "+ item.Y.ToString());
            }
            Clipboard.SetText(array.ToString(Newtonsoft.Json.Formatting.Indented));
        }
    }
}
