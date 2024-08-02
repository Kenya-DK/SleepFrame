using System;
using System.Windows.Forms;

namespace SleepFrame.Macros.Views
{
    public partial class TradingUserControl : UserControl
    {
        private TradingMacro _trading = null;
        public TradingUserControl(TradingMacro trading)
        {
            InitializeComponent();
            _trading = trading;

            foreach (var item in _trading.Messages)
                _dgvMessage.Rows.Add(item.Message, item.Enable);
        }

        private void BtnClear_Click(object sender, EventArgs e)
        {
            _trading.Messages.Clear();
            _dgvMessage.Rows.Clear();
        }

        private void BtnAdd_Click(object sender, EventArgs e)
        {
            if (textBox1.Text.Length > 0)
            {
                TradingMessage message = new TradingMessage(textBox1.Text, true);
                _dgvMessage.Rows.Add(message.Message, message.Enable);
                _trading.Messages.Add(message);
                textBox1.Text = string.Empty;
            }
        }

        private void DgvMessage_CellContentClick(object sender, DataGridViewCellEventArgs e)
        {
            if (_trading.Messages.Count <= e.RowIndex)
                return;
            switch (e.ColumnIndex)
            {
                case 1:
                    TradingMessage message = _trading.Messages[e.RowIndex];
                    if (message == null)
                        return;
                    message.Enable = !message.Enable;
                    break;
                default:
                    break;
            }
            _trading.SaveToFile();
        }

        private void DgvMessage_CellEndEdit(object sender, DataGridViewCellEventArgs e)
        {
            TradingMessage message = _trading.Messages[e.RowIndex];
            if (message != null)
                message.Message = (string)_dgvMessage.Rows[e.RowIndex].Cells[0].Value;
            _trading.SaveToFile();
        }

        private void DgvMessage_RowsRemoved(object sender, DataGridViewRowsRemovedEventArgs e)
        {
            if (_trading.Messages.Count <= e.RowIndex)
                return;
            _trading.Messages.RemoveAt(e.RowIndex);
        }
    }
}
