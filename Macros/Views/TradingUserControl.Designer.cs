namespace SleepFrame.Macros.Views
{
    partial class TradingUserControl
    {
        /// <summary> 
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary> 
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Component Designer generated code

        /// <summary> 
        /// Required method for Designer support - do not modify 
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.label1 = new System.Windows.Forms.Label();
            this._btnAdd = new System.Windows.Forms.Button();
            this._btnClear = new System.Windows.Forms.Button();
            this.textBox1 = new System.Windows.Forms.TextBox();
            this._dgvMessage = new System.Windows.Forms.DataGridView();
            this.chMessage = new System.Windows.Forms.DataGridViewTextBoxColumn();
            this.chEnable = new System.Windows.Forms.DataGridViewCheckBoxColumn();
            ((System.ComponentModel.ISupportInitialize)(this._dgvMessage)).BeginInit();
            this.SuspendLayout();
            // 
            // label1
            // 
            this.label1.Font = new System.Drawing.Font("Microsoft Sans Serif", 13.25F, System.Drawing.FontStyle.Bold, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.label1.Location = new System.Drawing.Point(3, 10);
            this.label1.Name = "label1";
            this.label1.Size = new System.Drawing.Size(664, 23);
            this.label1.TabIndex = 1;
            this.label1.Text = "Messages";
            this.label1.TextAlign = System.Drawing.ContentAlignment.MiddleCenter;
            // 
            // _btnAdd
            // 
            this._btnAdd.Location = new System.Drawing.Point(592, 158);
            this._btnAdd.Name = "_btnAdd";
            this._btnAdd.Size = new System.Drawing.Size(75, 23);
            this._btnAdd.TabIndex = 2;
            this._btnAdd.Text = "Add";
            this._btnAdd.UseVisualStyleBackColor = true;
            this._btnAdd.Click += new System.EventHandler(this.BtnAdd_Click);
            // 
            // _btnClear
            // 
            this._btnClear.Location = new System.Drawing.Point(7, 158);
            this._btnClear.Name = "_btnClear";
            this._btnClear.Size = new System.Drawing.Size(75, 23);
            this._btnClear.TabIndex = 2;
            this._btnClear.Text = "Clear";
            this._btnClear.UseVisualStyleBackColor = true;
            this._btnClear.Click += new System.EventHandler(this.BtnClear_Click);
            // 
            // textBox1
            // 
            this.textBox1.Location = new System.Drawing.Point(7, 187);
            this.textBox1.Name = "textBox1";
            this.textBox1.Size = new System.Drawing.Size(661, 20);
            this.textBox1.TabIndex = 3;
            // 
            // _dgvMessage
            // 
            this._dgvMessage.AllowUserToAddRows = false;
            this._dgvMessage.ColumnHeadersHeightSizeMode = System.Windows.Forms.DataGridViewColumnHeadersHeightSizeMode.AutoSize;
            this._dgvMessage.Columns.AddRange(new System.Windows.Forms.DataGridViewColumn[] {
            this.chMessage,
            this.chEnable});
            this._dgvMessage.Location = new System.Drawing.Point(7, 36);
            this._dgvMessage.Name = "_dgvMessage";
            this._dgvMessage.Size = new System.Drawing.Size(660, 116);
            this._dgvMessage.TabIndex = 4;
            this._dgvMessage.CellContentClick += new System.Windows.Forms.DataGridViewCellEventHandler(this.DgvMessage_CellContentClick);
            this._dgvMessage.CellEndEdit += new System.Windows.Forms.DataGridViewCellEventHandler(this.DgvMessage_CellEndEdit);
            this._dgvMessage.RowsRemoved += new System.Windows.Forms.DataGridViewRowsRemovedEventHandler(this.DgvMessage_RowsRemoved);
            // 
            // chMessage
            // 
            this.chMessage.AutoSizeMode = System.Windows.Forms.DataGridViewAutoSizeColumnMode.Fill;
            this.chMessage.HeaderText = "Message";
            this.chMessage.Name = "chMessage";
            // 
            // chEnable
            // 
            this.chEnable.AutoSizeMode = System.Windows.Forms.DataGridViewAutoSizeColumnMode.ColumnHeader;
            this.chEnable.HeaderText = "Enable";
            this.chEnable.Name = "chEnable";
            this.chEnable.Width = 46;
            // 
            // TradingUserControl
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.Controls.Add(this._dgvMessage);
            this.Controls.Add(this.textBox1);
            this.Controls.Add(this._btnClear);
            this.Controls.Add(this._btnAdd);
            this.Controls.Add(this.label1);
            this.Name = "TradingUserControl";
            this.Size = new System.Drawing.Size(670, 210);
            ((System.ComponentModel.ISupportInitialize)(this._dgvMessage)).EndInit();
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.Button _btnAdd;
        private System.Windows.Forms.Button _btnClear;
        private System.Windows.Forms.TextBox textBox1;
        private System.Windows.Forms.DataGridView _dgvMessage;
        private System.Windows.Forms.DataGridViewTextBoxColumn chMessage;
        private System.Windows.Forms.DataGridViewCheckBoxColumn chEnable;
    }
}
