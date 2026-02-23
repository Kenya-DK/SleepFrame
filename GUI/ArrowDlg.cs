using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SleepFrame
{
    public partial class ArrowDlg : Form
    {
        public ArrowDlg()
        {
            InitializeComponent();

            //BitmapRegion.CreateControlRegion(this, AutoClicker.Properties.Resources.flake);

        }

        private void ArrowDlg_Load(object sender, EventArgs e)
        {
            this.Size = new Size(17, 17);
        }
    }
}
