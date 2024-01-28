using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;
using static System.Windows.Forms.VisualStyles.VisualStyleElement.Menu;

namespace SleepFrame.Macros.Views
{
    public partial class TradingUserControl : UserControl
    {
        private Trading _trading = null;
        public TradingUserControl(Trading trading)
        {
            InitializeComponent();
            _trading = trading;
            foreach (var item in _trading.Messages)
                _lvMessages.Items.Add(new ListViewItem(item) { Tag = item });
        }

        private void _btnRemove_Click(object sender, EventArgs e)
        {
            foreach (ListViewItem item in _lvMessages.SelectedItems)
            {
                _lvMessages.Items.Remove(item);
                _trading.Messages.Remove(item.Tag.ToString());
            }
        }

        private void _btnClear_Click(object sender, EventArgs e)
        {
            _lvMessages.Items.Clear();
            _trading.Messages.Clear();
        }

        private void _btnAdd_Click(object sender, EventArgs e)
        {
            if (textBox1.Text.Length > 0)
            {
                _lvMessages.Items.Add(new ListViewItem(textBox1.Text) { Tag = textBox1.Text });
                _trading.Messages.Add(textBox1.Text);
            }
        }
    }
}
