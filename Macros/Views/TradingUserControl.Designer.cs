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
            this._lvMessages = new System.Windows.Forms.ListView();
            this._chMsg = ((System.Windows.Forms.ColumnHeader)(new System.Windows.Forms.ColumnHeader()));
            this.label1 = new System.Windows.Forms.Label();
            this._btnRemove = new System.Windows.Forms.Button();
            this._btnAdd = new System.Windows.Forms.Button();
            this._btnClear = new System.Windows.Forms.Button();
            this.textBox1 = new System.Windows.Forms.TextBox();
            this.SuspendLayout();
            // 
            // _lvMessages
            // 
            this._lvMessages.Columns.AddRange(new System.Windows.Forms.ColumnHeader[] {
            this._chMsg});
            this._lvMessages.GridLines = true;
            this._lvMessages.HideSelection = false;
            this._lvMessages.Location = new System.Drawing.Point(4, 36);
            this._lvMessages.Name = "_lvMessages";
            this._lvMessages.Size = new System.Drawing.Size(663, 97);
            this._lvMessages.TabIndex = 0;
            this._lvMessages.UseCompatibleStateImageBehavior = false;
            this._lvMessages.View = System.Windows.Forms.View.Details;
            // 
            // _chMsg
            // 
            this._chMsg.Text = "Message";
            this._chMsg.Width = 650;
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
            // _btnRemove
            // 
            this._btnRemove.Location = new System.Drawing.Point(6, 140);
            this._btnRemove.Name = "_btnRemove";
            this._btnRemove.Size = new System.Drawing.Size(75, 23);
            this._btnRemove.TabIndex = 2;
            this._btnRemove.Text = "Remove";
            this._btnRemove.UseVisualStyleBackColor = true;
            this._btnRemove.Click += new System.EventHandler(this._btnRemove_Click);
            // 
            // _btnAdd
            // 
            this._btnAdd.Location = new System.Drawing.Point(592, 139);
            this._btnAdd.Name = "_btnAdd";
            this._btnAdd.Size = new System.Drawing.Size(75, 23);
            this._btnAdd.TabIndex = 2;
            this._btnAdd.Text = "Add";
            this._btnAdd.UseVisualStyleBackColor = true;
            this._btnAdd.Click += new System.EventHandler(this._btnAdd_Click);
            // 
            // _btnClear
            // 
            this._btnClear.Location = new System.Drawing.Point(298, 139);
            this._btnClear.Name = "_btnClear";
            this._btnClear.Size = new System.Drawing.Size(75, 23);
            this._btnClear.TabIndex = 2;
            this._btnClear.Text = "Clear";
            this._btnClear.UseVisualStyleBackColor = true;
            this._btnClear.Click += new System.EventHandler(this._btnClear_Click);
            // 
            // textBox1
            // 
            this.textBox1.Location = new System.Drawing.Point(6, 170);
            this.textBox1.Name = "textBox1";
            this.textBox1.Size = new System.Drawing.Size(661, 20);
            this.textBox1.TabIndex = 3;
            // 
            // TradingUserControl
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.Controls.Add(this.textBox1);
            this.Controls.Add(this._btnClear);
            this.Controls.Add(this._btnAdd);
            this.Controls.Add(this._btnRemove);
            this.Controls.Add(this.label1);
            this.Controls.Add(this._lvMessages);
            this.Name = "TradingUserControl";
            this.Size = new System.Drawing.Size(670, 210);
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.ListView _lvMessages;
        private System.Windows.Forms.ColumnHeader _chMsg;
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.Button _btnRemove;
        private System.Windows.Forms.Button _btnAdd;
        private System.Windows.Forms.Button _btnClear;
        private System.Windows.Forms.TextBox textBox1;
    }
}
